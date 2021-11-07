mod app;
mod parse;
mod sampler;

use tokio::io::{stdout, AsyncWriteExt as _};
use zysfs::tokio::Read as ZysfsRead;

use crate::cli::app::*;
use crate::profile::{Error as ProfileError, Profile};
use crate::{logging, Chain, Error, Result};

// Command-line interface.
#[derive(Clone, Debug)]
pub struct Cli {
    quiet: Option<()>,
    show_cpu: Option<()>,
    show_drm: Option<()>,
    #[cfg(feature = "nvml")]
    show_nvml: Option<()>,
    show_pstate: Option<()>,
    show_rapl: Option<()>,
    profile: Option<Profile>,
    chains: Vec<Chain>,
}

impl Cli {
    // Create a new instance for the given argv.
    pub async fn new(argv: &[String]) -> Result<Self> {
        log::debug!("Profile config paths: {:#?}", Profile::paths().await);
        let p = parse::Parser::new(argv)?;
        let mut chains = vec![];
        let profile = if let Some(pr) = p.str(ARG_PROFILE) {
            let pr = Profile::new(pr).await?;
            let mut c = pr.read().await?;
            if c.has_values() {
                c.resolve().await;
                chains.push(c);
            }
            Some(pr)
        } else {
            None
        };
        let mut c = Chain::try_from(&p)?;
        if c.has_values() {
            c.resolve().await;
            chains.push(c);
        }
        let s = Self {
            quiet: p.flag(ARG_QUIET),
            show_cpu: p.flag(ARG_SHOW_CPU),
            show_drm: p.flag(ARG_SHOW_DRM),
            #[cfg(feature = "nvml")]
            show_nvml: p.flag(ARG_SHOW_NVML),
            show_pstate: p.flag(ARG_SHOW_PSTATE),
            show_rapl: p.flag(ARG_SHOW_RAPL),
            profile,
            chains,
        };
        Ok(s)
    }

    // Return true if --show-* args are present.
    #[allow(clippy::let_and_return)]
    fn has_show_args(&self) -> bool {
        let b = self.show_cpu.is_some()
            || self.show_drm.is_some()
            || self.show_pstate.is_some()
            || self.show_rapl.is_some();
        #[cfg(feature = "nvml")]
        let b = b || self.show_nvml.is_some();
        b
    }

    // Print values tables.
    async fn print_values(&self, samplers: &sampler::Samplers) -> Result<()> {
        use crate::format::FormatValues as _;
        let mut buf = Vec::with_capacity(3000);
        let show_all = !self.has_show_args();
        if show_all || self.show_cpu.is_some() {
            if let Some(cpu) = zysfs::cpu::Cpu::read(()).await {
                if let Some(cpufreq) = zysfs::cpufreq::Cpufreq::read(()).await {
                    (cpu, cpufreq).format_values(&mut buf, ()).await?;
                }
            }
        }
        if show_all || self.show_pstate.is_some() {
            if let Some(intel_pstate) = zysfs::intel_pstate::IntelPstate::read(()).await {
                intel_pstate.format_values(&mut buf, ()).await?;
            }
        }
        if show_all || self.show_rapl.is_some() {
            if let Some(intel_rapl) = zysfs::intel_rapl::IntelRapl::read(()).await {
                intel_rapl
                    .format_values(&mut buf, samplers.clone().into_samplers())
                    .await?;
            }
        }
        if show_all || self.show_drm.is_some() {
            if let Some(drm) = zysfs::drm::Drm::read(()).await {
                drm.format_values(&mut buf, ()).await?;
            }
        }
        #[cfg(feature = "nvml")]
        if show_all || self.show_nvml.is_some() {
            nvml_facade::Nvml.format_values(&mut buf, ()).await?;
        }
        let s = String::from_utf8_lossy(&buf);
        let mut stdout = stdout();
        stdout.write_all(s[..s.len() - 1].as_bytes()).await.unwrap();
        stdout.flush().await.unwrap();
        Ok(())
    }

    // Command-line interface app logic.
    pub async fn run(&self) -> Result<()> {
        let mut samplers = sampler::Samplers::start(self).await;
        for chain in &self.chains {
            chain.apply().await;
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
        let r = self.print_values(&samplers).await;
        samplers.stop().await;
        r?;
        Ok(())
    }
}

// Cli application.
#[derive(Clone, Debug)]
pub struct App;

impl App {
    // Run app with args.
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

    // Run app.
    pub async fn run() {
        let args: Vec<String> = std::env::args().collect();
        Self::run_with_args(&args).await
    }
}
