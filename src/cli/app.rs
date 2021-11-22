use tokio::io::{stdout, AsyncWriteExt as _};

#[cfg(feature = "nvml")]
use crate::cli::parser::ARG_SHOW_NV;
use crate::cli::parser::{
    Parser,
    ARG_PROFILE,
    ARG_QUIET,
    ARG_SHOW_CPU,
    ARG_SHOW_I915,
    ARG_SHOW_PSTATE,
    ARG_SHOW_RAPL,
};
use crate::cli::profile::path::config_paths;
use crate::cli::profile::{Error as ProfileError, Profile};
use crate::cli::sampler::Samplers;
use crate::cli::values::Values;
use crate::cli::{format, logging, Error, Result};
use crate::{Machine, Values as _};

#[derive(Clone, Debug)]
pub struct Cli {
    pub(in crate::cli) quiet: Option<()>,
    pub(in crate::cli) show_cpu: Option<()>,
    pub(in crate::cli) show_i915: Option<()>,
    #[cfg(feature = "nvml")]
    pub(in crate::cli) show_nvml: Option<()>,
    pub(in crate::cli) show_pstate: Option<()>,
    pub(in crate::cli) show_rapl: Option<()>,
    pub(in crate::cli) profile: Option<Profile>,
    pub(in crate::cli) values: Vec<Values>,
}

impl Cli {
    pub async fn new(argv: &[String]) -> Result<Self> {
        log::debug!("Profile config paths: {:#?}", config_paths().await);
        let p = Parser::new(argv)?;
        let mut values = vec![];
        let profile = if let Some(pr) = p.str(ARG_PROFILE) {
            let pr = Profile::new(pr).await?;
            for mut v in pr.values().await? {
                if v.has_values() {
                    v.resolve().await;
                    values.push(v);
                }
            }
            Some(pr)
        } else {
            None
        };
        for mut v in Vec::try_from(&p)? {
            if v.has_values() {
                v.resolve().await;
                values.push(v);
            }
        }
        let quiet = p.flag(ARG_QUIET);
        let show_cpu = p.flag(ARG_SHOW_CPU);
        let show_i915 = p.flag(ARG_SHOW_I915);
        #[cfg(feature = "nvml")]
        let show_nvml = p.flag(ARG_SHOW_NV);
        let show_pstate = p.flag(ARG_SHOW_PSTATE);
        let show_rapl = p.flag(ARG_SHOW_RAPL);
        let s = Self {
            quiet,
            show_cpu,
            show_i915,
            #[cfg(feature = "nvml")]
            show_nvml,
            show_pstate,
            show_rapl,
            profile,
            values,
        };
        Ok(s)
    }

    #[allow(clippy::let_and_return)]
    pub(in crate::cli) fn has_show_args(&self) -> bool {
        let b = self.show_cpu.is_some()
            || self.show_i915.is_some()
            || self.show_pstate.is_some()
            || self.show_rapl.is_some();
        #[cfg(feature = "nvml")]
        let b = b || self.show_nvml.is_some();
        b
    }

    async fn print(&self, samplers: &Samplers) -> Result<()> {
        let machine = if let Some(s) = Machine::read(()).await {
            s
        } else {
            return Ok(());
        };
        let mut buf = Vec::with_capacity(3000);
        let show_all = !self.has_show_args();
        if show_all || self.show_cpu.is_some() {
            format::cpu(&mut buf, &machine).await.unwrap();
        }
        if show_all || self.show_pstate.is_some() {
            format::intel_pstate(&mut buf, &machine).await.unwrap();
        }
        if show_all || self.show_rapl.is_some() {
            format::intel_rapl(&mut buf, &machine, samplers.clone().into_samplers())
                .await
                .unwrap();
        }
        if show_all || self.show_i915.is_some() {
            format::i915(&mut buf, &machine).await.unwrap();
        }
        #[cfg(feature = "nvml")]
        if show_all || self.show_nvml.is_some() {
            format::nvml(&mut buf, &machine).await.unwrap();
        }
        let s = String::from_utf8_lossy(&buf);
        let mut stdout = stdout();
        stdout.write_all(s[..s.len() - 1].as_bytes()).await.unwrap();
        stdout.flush().await.unwrap();
        Ok(())
    }

    pub async fn run(&self) -> Result<()> {
        let mut samplers = Samplers::start(self).await;
        for v in &self.values {
            let m = Machine::from(v);
            m.write().await;
        }
        if let Some(p) = self.profile.as_ref() {
            let r = p.set_recent().await;
            if r.is_err() {
                samplers.stop().await;
                r?;
            }
        }
        if self.quiet.is_none() {
            samplers.wait().await;
            let r = self.print(&samplers).await;
            samplers.stop().await;
            r?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct App;

impl App {
    pub async fn run_with_args(args: &[String]) {
        logging::configure().await;
        match Cli::new(args).await {
            Ok(cli) => match cli.run().await {
                Ok(()) => std::process::exit(0),
                Err(e) => {
                    log::error!("Error: {}", e);
                    std::process::exit(1);
                },
            },
            Err(e) => match &e {
                Error::Clap(e) => {
                    if let clap::ErrorKind::HelpDisplayed = e.kind {
                        let mut s = stdout();
                        s.write_all(e.message.as_bytes()).await.unwrap();
                        s.write_all("\n".as_bytes()).await.unwrap();
                        s.flush().await.unwrap();
                        std::process::exit(0);
                    } else {
                        log::error!("{}", e);
                        std::process::exit(1);
                    }
                },
                Error::Profile(ProfileError::StateCorrupt { path }) => {
                    use tokio::fs::remove_file;
                    log::error!(
                        "Error: profile state file corrupted, removing {}",
                        path.display()
                    );
                    match remove_file(path).await {
                        Ok(()) => std::process::exit(1),
                        Err(e) => {
                            log::error!("Error: {}", e);
                            std::process::exit(1);
                        },
                    }
                },
                _ => {
                    log::error!("Error: {}", e);
                    std::process::exit(1);
                },
            },
        }
    }

    pub async fn run() {
        let args: Vec<String> = std::env::args().collect();
        Self::run_with_args(&args).await
    }
}
