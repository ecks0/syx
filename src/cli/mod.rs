mod app;
mod chain;
mod counter;
mod env;
mod lazy;
mod logging;
mod parser;
mod path;
mod profile;
mod sampler;

pub use profile::Error as ProfileError;

use zysfs::types::{self as sysfs, tokio::Read as SysfsRead};
use std::{convert::TryFrom};
use crate::{Chain, Error, Knobs, Result};
use crate::parse::{BoolStr, CardIds, DurationStr, FrequencyStr, Indices, PowerStr, Toggles};
use tokio::io::{AsyncWriteExt, stdout};

const NAME: &str                  = "knobs";

const ARG_QUIET: &str             = "quiet";

const ARG_SHOW_CPU: &str          = "show-cpu";
const ARG_SHOW_DRM: &str          = "show-drm";
#[cfg(feature = "nvml")]
const ARG_SHOW_NVML: &str         = "show-nvml";
const ARG_SHOW_PSTATE: &str       = "show-pstate";
const ARG_SHOW_RAPL: &str         = "show-rapl";

const ARG_CPU: &str               = "cpu";
const ARG_CPU_ON: &str            = "cpu-on";
const ARG_CPU_ON_EACH: &str       = "cpu-on-each";

const ARG_CPUFREQ_GOV: &str       = "cpufreq-gov";
const ARG_CPUFREQ_MIN: &str       = "cpufreq-min";
const ARG_CPUFREQ_MAX: &str       = "cpufreq-max";

const ARG_DRM_I915: &str          = "drm-i915";
const ARG_DRM_I915_MIN: &str      = "drm-i915-min";
const ARG_DRM_I915_MAX: &str      = "drm-i915-max";
const ARG_DRM_I915_BOOST: &str    = "drm-i915-boost";

#[cfg(feature = "nvml")]
const ARG_NVML: &str              = "nvml";
#[cfg(feature = "nvml")]
const ARG_NVML_GPU_MIN: &str      = "nvml-gpu-min";
#[cfg(feature = "nvml")]
const ARG_NVML_GPU_MAX: &str      = "nvml-gpu-max";
#[cfg(feature = "nvml")]
const ARG_NVML_GPU_RESET: &str    = "nvml-gpu-reset";
#[cfg(feature = "nvml")]
const ARG_NVML_POWER_LIMIT: &str  = "nvml-power-limit";

const ARG_PSTATE_EPB: &str        = "pstate-epb";
const ARG_PSTATE_EPP: &str        = "pstate-epp";

const ARG_RAPL_PACKAGE: &str      = "rapl-package";
const ARG_RAPL_ZONE: &str         = "rapl-zone";
const ARG_RAPL_LONG_LIMIT: &str   = "rapl-long-limit";
const ARG_RAPL_LONG_WINDOW: &str  = "rapl-long-window";
const ARG_RAPL_SHORT_LIMIT: &str  = "rapl-short-limit";
const ARG_RAPL_SHORT_WINDOW: &str = "rapl-short-window";

const ARG_PROFILE: &str           = "PROFILE";

const ARG_CHAIN: &str             = "CHAIN";

const AFTER_HELP: &str = r#"            BOOL   0, 1, true, false
             IDS   comma-delimited sequence of integers and/or integer ranges
           HERTZ*  mhz when unspecified: hz/h - khz/k - mhz/m - ghz/g - thz/t
            SECS   ms when unspecified: ns/n - us/u - ms/m - s
         TOGGLES   sequence of 0 (off), 1 (on), or _ (skip), where position denotes id
           WATTS*  mw when unspecified: uw/u - mw/m - w - kw/k

        * Floating point values may be given for these units.

    Values for supported hardware are shown unless the --show-* or --quiet flags are used.

    All flags may be expressed as env vars. For example:

        --show-cpu                 → KNOBS_SHOW_CPU=1
        --cpu 1,3-5                → KNOBS_CPU=1,3-5
        --drm-i915-boost 1.2ghz    → KNOBS_DRM_I915_BOOST=1.2ghz

    The KNOBS_LOG env var may be set to trace, debug, info, warn, or error (default).
"#;

// Build a `Knobs` from a `Parser`.
impl<'a> TryFrom<parser::Parser<'a>> for Knobs {
    type Error = Error;

    fn try_from(p: parser::Parser<'a>) -> Result<Self> {
        let s = Self {
            cpu: p.from_str_as::<Indices, _>(ARG_CPU)?,
            cpu_on: p.from_str_as::<BoolStr, _>(ARG_CPU_ON)?,
            cpu_on_each: p.from_str_as::<Toggles, _>(ARG_CPU_ON_EACH)?,
            cpufreq_gov: p.str(ARG_CPUFREQ_GOV),
            cpufreq_min: p.from_str_as::<FrequencyStr, _>(ARG_CPUFREQ_MIN)?,
            cpufreq_max: p.from_str_as::<FrequencyStr, _>(ARG_CPUFREQ_MAX)?,
            drm_i915: p.from_str_as::<CardIds, _>(ARG_DRM_I915)?,
            drm_i915_min: p.from_str_as::<FrequencyStr, _>(ARG_DRM_I915_MIN)?,
            drm_i915_max: p.from_str_as::<FrequencyStr, _>(ARG_DRM_I915_MAX)?,
            drm_i915_boost: p.from_str_as::<FrequencyStr, _>(ARG_DRM_I915_BOOST)?,
            #[cfg(feature = "nvml")]
            nvml: p.from_str_as::<CardIds, _>(ARG_NVML)?,
            #[cfg(feature = "nvml")]
            nvml_gpu_min: p.from_str_as::<FrequencyStr, _>(ARG_NVML_GPU_MIN)?,
            #[cfg(feature = "nvml")]
            nvml_gpu_max: p.from_str_as::<FrequencyStr, _>(ARG_NVML_GPU_MAX)?,
            #[cfg(feature = "nvml")]
            nvml_gpu_reset: p.flag(ARG_NVML_GPU_RESET).map(|_| true),
            #[cfg(feature = "nvml")]
            nvml_power_limit: p.from_str_as::<PowerStr, _>(ARG_NVML_POWER_LIMIT)?,
            pstate_epb: p.int::<u64>(ARG_PSTATE_EPB)?,
            pstate_epp: p.str(ARG_PSTATE_EPP),
            rapl_package: p.int::<u64>(ARG_RAPL_PACKAGE)?,
            rapl_zone: p.int::<u64>(ARG_RAPL_ZONE)?,
            rapl_long_limit: p.from_str_as::<PowerStr, _>(ARG_RAPL_LONG_LIMIT)?,
            rapl_long_window: p.from_str_as::<DurationStr, _>(ARG_RAPL_LONG_WINDOW)?,
            rapl_short_limit: p.from_str_as::<PowerStr, _>(ARG_RAPL_SHORT_LIMIT)?,
            rapl_short_window: p.from_str_as::<DurationStr, _>(ARG_RAPL_SHORT_WINDOW)?,
        };
        Ok(s)
   }
}

// Command-line interface.
#[derive(Clone, Debug)]
pub struct Cli {
    pub quiet: Option<()>,
    pub show_cpu: Option<()>,
    pub show_drm: Option<()>,
    #[cfg(feature = "nvml")]
    pub show_nvml: Option<()>,
    pub show_pstate: Option<()>,
    pub show_rapl: Option<()>,
    pub chains: Vec<Chain>
}

impl Cli {
    // Create a new instance for the given argv.
    pub async fn new(argv: &[String]) -> Result<Self> {
        let p = parser::Parser::new(argv)?;
        let mut chains = vec![];
        if let Some(name) = p.str(ARG_PROFILE) {
            let c = profile::read(&name).await?;
            let c = chain::resolve_chain(c).await?;
            chains.push(c);
        }
        chains.push(chain::resolve_parser(p.clone()).await?);
        let s = Self {
            quiet: p.flag(ARG_QUIET),
            show_cpu: p.flag(ARG_SHOW_CPU),
            show_drm: p.flag(ARG_SHOW_DRM),
            #[cfg(feature = "nvml")]
            show_nvml: p.flag(ARG_SHOW_NVML),
            show_pstate: p.flag(ARG_SHOW_PSTATE),
            show_rapl: p.flag(ARG_SHOW_RAPL),
            chains,
        };
        Ok(s)
    }

    // Return true if --show-* args are present.
    fn has_show_args(&self) -> bool {
        let b = self.show_cpu.is_some() ||
            self.show_drm.is_some() ||
            self.show_pstate.is_some() ||
            self.show_rapl.is_some();
        #[cfg(feature = "nvml")]
        let b = b || self.show_nvml.is_some();
        b
    }

    // Print values tables.
    async fn print_values(&self, samplers: sampler::Samplers) -> Result<()> {
        use crate::format::FormatValues as _;
        let mut buf = Vec::with_capacity(3000);
        let show_all = !self.has_show_args();
        if show_all || self.show_cpu.is_some() {
            if let Some(cpu) = sysfs::cpu::Cpu::read(()).await {
                if let Some(cpufreq) = sysfs::cpufreq::Cpufreq::read(()).await {
                    (cpu, cpufreq).format_values(&mut buf, ()).await?;
                }
            }
        }
        if show_all || self.show_pstate.is_some() {
            if let Some(intel_pstate) = sysfs::intel_pstate::IntelPstate::read(()).await {
                intel_pstate.format_values(&mut buf, ()).await?;
            }
        }
        if show_all || self.show_rapl.is_some() {
            if let Some(intel_rapl) = sysfs::intel_rapl::IntelRapl::read(()).await {
                intel_rapl.format_values(&mut buf, samplers.into_option()).await?;
            }
        }
        if show_all || self.show_drm.is_some() {
            if let Some(drm) = sysfs::drm::Drm::read(()).await {
                drm.format_values(&mut buf, ()).await?;
            }
        }
        #[cfg(feature = "nvml")]
        if show_all || self.show_nvml.is_some() {
            nvml_facade::Nvml.format_values(&mut buf, ()).await?;
        }
        let s = String::from_utf8_lossy(&buf);
        let mut stdout = stdout();
        stdout.write_all(s[..s.len()-1].as_bytes()).await?;
        stdout.flush().await?;
        Ok(())
    }

    // Command-line interface app logic.
    pub async fn run(&self) -> Result<()> {
        let mut samplers = sampler::Samplers::new(self).await;
        let begin = counter::delta().await;
        for chain in &self.chains { chain.apply_values().await; }
        if self.quiet.is_some() { return Ok(()); } // samplers will not start if quiet
        samplers.wait(begin).await;
        let pr = self.print_values(samplers.clone()).await;
        samplers.stop().await;
        pr?;
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
            Ok(cli) =>
                if let Err(e) = cli.run().await {
                    log::error!("Error: {}", e);
                    std::process::exit(2);
                }
            Err(e) => {
                if let Error::Clap(e) = &e {
                    if let clap::ErrorKind::HelpDisplayed = e.kind {
                        let mut s = stdout();
                        s.write_all(e.message.as_bytes()).await.unwrap();
                        s.write_all("\n".as_bytes()).await.unwrap();
                        s.flush().await.unwrap();
                        std::process::exit(0);
                    }
                }
                log::error!("Error: {}", e);
                std::process::exit(1);
            },
        }
    }

    // Run app.
    pub async fn run() {
        let args: Vec<String> = std::env::args().collect();
        Self::run_with_args(&args).await
    }
}
