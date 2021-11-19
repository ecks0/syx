use tokio::io::{stdout, AsyncWriteExt as _};

use crate::cli::group::Groups;
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
use crate::cli::{format, logging, Error, Result};
use crate::{Resource as _, System};

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
    pub(in crate::cli) groups: Vec<Groups>,
}

impl Cli {
    pub async fn new(argv: &[String]) -> Result<Self> {
        log::debug!("Profile config paths: {:#?}", config_paths().await);
        let p = Parser::new(argv)?;
        let mut groups = vec![];
        let profile = if let Some(pr) = p.str(ARG_PROFILE) {
            let pr = Profile::new(pr).await?;
            let mut g = pr.groups().await?;
            if g.has_values() {
                g.resolve().await;
                groups.push(g);
            }
            Some(pr)
        } else {
            None
        };
        let mut g = Groups::try_from(&p)?;
        if g.has_values() {
            g.resolve().await;
            groups.push(g);
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
            groups,
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
        let system = if let Some(s) = System::read(()).await {
            s
        } else {
            return Ok(());
        };
        let mut buf = Vec::with_capacity(3000);
        let show_all = !self.has_show_args();
        if show_all || self.show_cpu.is_some() {
            let _ = format::cpu(&mut buf, &system).await;
        }
        if show_all || self.show_pstate.is_some() {
            let _ = format::intel_pstate(&mut buf, &system).await;
        }
        if show_all || self.show_rapl.is_some() {
            let _ = format::intel_rapl(&mut buf, &system, samplers.clone().into_samplers()).await;
        }
        if show_all || self.show_i915.is_some() {
            let _ = format::i915(&mut buf, &system).await;
        }
        #[cfg(feature = "nvml")]
        if show_all || self.show_nvml.is_some() {
            let _ = format::nvml(&mut buf, &system).await;
        }
        let s = String::from_utf8_lossy(&buf);
        let mut stdout = stdout();
        stdout.write_all(s[..s.len() - 1].as_bytes()).await.unwrap();
        stdout.flush().await.unwrap();
        Ok(())
    }

    pub async fn run(&self) -> Result<()> {
        let mut samplers = Samplers::start(self).await;
        for g in &self.groups {
            g.apply().await;
        }
        if let Some(p) = self.profile.as_ref() {
            let r = p.set_recent().await;
            if r.is_err() {
                samplers.stop().await;
                r?;
            }
        }
        if self.quiet.is_some() {
            return Ok(());
        } // samplers do not start when quiet
        samplers.wait().await;
        let r = self.print(&samplers).await;
        samplers.stop().await;
        r?;
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
