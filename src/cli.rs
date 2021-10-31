use zysfs::types::{self as sysfs, tokio::Read as _};
use std::{convert::TryFrom, str::FromStr, time::{Duration, Instant}};
use tokio::{sync::OnceCell, time::sleep};
use crate::{Error, Result, data::{RaplSampler, RaplSamplers}};

const NAME: &str                  = "knobs";

const ARG_QUIET: &str             = "quiet";

const ARG_SHOW_CPU: &str          = "show-cpu";
const ARG_SHOW_DRM: &str          = "show-drm";
#[cfg(feature = "nvml")]
const ARG_SHOW_NVML: &str         = "show-nvml";
const ARG_SHOW_PSTATE: &str       = "show-pstate";
const ARG_SHOW_RAPL: &str         = "show-rapl";

const ARG_CPU: &str               = "cpu";
const ARG_CPU_ONLINE: &str        = "cpu-online";

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

const AFTER_HELP: &str = r#"            BOOL   0, 1, true, false
             IDS   comma-delimited sequence of integers and/or integer ranges
           HERTZ*  mhz when unspecified: hz/h - khz/k - mhz/m - ghz/g - thz/t
            SECS   ms when unspecified: ns/n - us/u - ms/m - s
           WATTS*  mw when unspecified: uw/u - mw/m - w - kw/k

        * Floating point values may be given for these units.

    Values for supported hardware are shown unless the --show-* or --quiet flags are used.

    All flags may be expressed as env vars. For example:

        --show-cpu                 → KNOBS_SHOW_CPU=1
        --cpu 1,3-5                → KNOBS_CPU=1,3-5
        --drm-i915-boost 1.2ghz    → KNOBS_DRM_I915_BOOST=1.2ghz

    The KNOBS_LOG env var may be set to trace, debug, info, warn, or error (default).
"#;

// Convert a cli flag name to an environment variable name.
fn env_name(cli_name: &str) -> String {
    format!("KNOBS_{}", cli_name.to_uppercase().replace("-", "_"))
}

// Return the environment variable value, if any, for the given cli flag name.
fn env_value(cli_name: &str) -> Option<String> {
    match std::env::var(&env_name(cli_name)) {
        Ok(v) => {
            log::debug!("--{}: using value from environment: {}", cli_name, v);
            Some(v)
        },
        _ => None,
    }
}

// Return a flag value, preferring the command line, falling back to environment variables.
fn flag(name: &str, m: &clap::ArgMatches) -> Option<()> {
    match m.is_present(name) {
        true => Some(()),
        false =>
            match env_value(&env_name(name))
                .map(|v| !v.is_empty() && v != "0" && v.to_lowercase() != "false")
                .unwrap_or(false)
            {
                true => Some(()),
                false => None,
            },
    }
}

// Parse and return an argument value, preferring the command line, falling back to environment variables.
fn arg<T: FromStr<Err = crate::Error>>(name: &str, m: &clap::ArgMatches) -> Result<Option<T>> {
    match m.value_of(name)
        .map(String::from)
        .or_else(|| env_value(name))
    {
        Some(v) => Ok(Some(
            T::from_str(&v)
                .map_err(|e| Error::parse_flag(name, e.to_string()))?
        )),
        None => Ok(None),
    }
}

// Parse and return an int argument value, preferring the command line, falling back to environment variables.
fn arg_int<T: FromStr<Err = std::num::ParseIntError>>(name: &str, m: &clap::ArgMatches) -> Result<Option<T>> {
    match m.value_of(name)
        .map(|v| v.to_string())
        .or_else(|| env_value(name))
    {
        Some(v) => Ok(Some(
            T::from_str(&v)
                .map_err(|_| Error::parse_flag(name, "Expected integer value".into()))?
        )),
        None => Ok(None),
    }
}

// Parse and return a string argument value, preferring the command line, falling back to environment variables.
fn arg_str(name: &str, m: &clap::ArgMatches) -> Option<String> {
    m.value_of(name)
        .map(|v| v.to_string())
        .or_else(|| env_value(name))
}

impl<'a> TryFrom<clap::ArgMatches<'a>> for crate::Knobs {
    type Error = Error;

    fn try_from(m: clap::ArgMatches<'a>) -> Result<Self> {
        let s = Self {
            cpu: arg::<crate::Indices>(ARG_CPU, &m)?.map(|v| v.into()),
            cpu_online: arg::<crate::BoolStr>(ARG_CPU_ONLINE, &m)?.map(|v| v.into()),
            cpufreq_gov: arg_str(ARG_CPUFREQ_GOV, &m),
            cpufreq_min: arg::<crate::FrequencyStr>(ARG_CPUFREQ_MIN, &m)?.map(|v| v.into()),
            cpufreq_max: arg::<crate::FrequencyStr>(ARG_CPUFREQ_MAX, &m)?.map(|v| v.into()),
            drm_i915: arg::<crate::CardIds>(ARG_DRM_I915, &m)?.map(|v| v.into()),
            drm_i915_min: arg::<crate::FrequencyStr>(ARG_DRM_I915_MIN, &m)?.map(|v| v.into()),
            drm_i915_max: arg::<crate::FrequencyStr>(ARG_DRM_I915_MAX, &m)?.map(|v| v.into()),
            drm_i915_boost: arg::<crate::FrequencyStr>(ARG_DRM_I915_BOOST, &m)?.map(|v| v.into()),
            #[cfg(feature = "nvml")]
            nvml: arg::<crate::CardIds>(ARG_NVML, &m)?.map(|v| v.into()),
            #[cfg(feature = "nvml")]
            nvml_gpu_min: arg::<crate::FrequencyStr>(ARG_NVML_GPU_MIN, &m)?.map(|v| v.into()),
            #[cfg(feature = "nvml")]
            nvml_gpu_max: arg::<crate::FrequencyStr>(ARG_NVML_GPU_MAX, &m)?.map(|v| v.into()),
            #[cfg(feature = "nvml")]
            nvml_gpu_reset: flag(ARG_NVML_GPU_RESET, &m).map(|_| true),
            #[cfg(feature = "nvml")]
            nvml_power_limit: arg::<crate::PowerStr>(ARG_NVML_POWER_LIMIT, &m)?.map(|v| v.into()),
            pstate_epb: arg_int::<u64>(ARG_PSTATE_EPB, &m)?,
            pstate_epp: arg_str(ARG_PSTATE_EPP, &m),
            rapl_package: arg_int::<u64>(ARG_RAPL_PACKAGE, &m)?,
            rapl_zone: arg_int::<u64>(ARG_RAPL_ZONE, &m)?,
            rapl_long_limit: arg::<crate::PowerStr>(ARG_RAPL_LONG_LIMIT, &m)?.map(|v| v.into()),
            rapl_long_window: arg::<crate::DurationStr>(ARG_RAPL_LONG_WINDOW, &m)?.map(|v| v.into()),
            rapl_short_limit: arg::<crate::PowerStr>(ARG_RAPL_SHORT_LIMIT, &m)?.map(|v| v.into()),
            rapl_short_window: arg::<crate::DurationStr>(ARG_RAPL_SHORT_WINDOW, &m)?.map(|v| v.into()),
        };
        Ok(s)
   }
}

#[derive(Clone, Debug)]
pub struct Cli {
    pub quiet: Option<()>,
    pub show_cpu: Option<()>,
    pub show_drm: Option<()>,
    #[cfg(feature = "nvml")]
    pub show_nvml: Option<()>,
    pub show_pstate: Option<()>,
    pub show_rapl: Option<()>,
    pub chain: crate::Chain,
}

impl Cli {

    // Build and return a clap app.
    fn app(argv0: &str) -> clap::App {
        use clap::{App, AppSettings, Arg, crate_version};

        let a = App::new(argv0)

            .setting(AppSettings::DeriveDisplayOrder)
            .setting(AppSettings::DisableHelpSubcommand)
            .setting(AppSettings::DisableVersion)
            .setting(AppSettings::TrailingVarArg)
            .setting(AppSettings::UnifiedHelpMessage)

            .version(crate_version!())

            .after_help(AFTER_HELP)

            .arg(Arg::with_name(ARG_QUIET)
                .short("q")
                .long(ARG_QUIET)
                .takes_value(false)
                .help("Do not print values"))

            .arg(Arg::with_name(ARG_SHOW_CPU)
                .long(ARG_SHOW_CPU)
                .takes_value(false)
                .help("Print cpu and cpufreq values"))

            .arg(Arg::with_name(ARG_SHOW_DRM)
                .long(ARG_SHOW_DRM)
                .takes_value(false)
                .help("Print drm values"));

        #[cfg(feature = "nvml")]
        let a = a

            .arg(Arg::with_name(ARG_SHOW_NVML)
                .long(ARG_SHOW_NVML)
                .takes_value(false)
                .help("Print nvidia management values"));

        let a = a

            .arg(Arg::with_name(ARG_SHOW_PSTATE)
                .long(ARG_SHOW_PSTATE)
                .takes_value(false)
                .help("Print intel-pstate values"))

            .arg(Arg::with_name(ARG_SHOW_RAPL)
                .long(ARG_SHOW_RAPL)
                .takes_value(false)
                .help("Print intel-rapl values"))

            .arg(Arg::with_name(ARG_CPU)
                .short("c")
                .long(ARG_CPU)
                .takes_value(true)
                .value_name("IDS")
                .help("Target cpu ids, default all, ex. 0,1,3-5"))

            .arg(Arg::with_name(ARG_CPU_ONLINE)
                .short("o")
                .long(ARG_CPU_ONLINE)
                .takes_value(true)
                .value_name("BOOL")
                .help("Set cpu online status per --cpu"))

            .arg(Arg::with_name(ARG_CPUFREQ_GOV)
                .short("g")
                .long(ARG_CPUFREQ_GOV)
                .takes_value(true)
                .value_name("STR")
                .help("Set cpufreq governor per --cpu"))

            .arg(Arg::with_name(ARG_CPUFREQ_MIN)
                .short("n")
                .long(ARG_CPUFREQ_MIN)
                .takes_value(true)
                .value_name("HERTZ")
                .help("Set cpufreq min freq per --cpu, ex. 1200 or 1.2ghz"))

            .arg(Arg::with_name(ARG_CPUFREQ_MAX)
                .short("x")
                .long(ARG_CPUFREQ_MAX)
                .takes_value(true)
                .value_name("HERTZ")
                .help("Set cpufreq max freq per --cpu, ex. 1200 or 1.2ghz"))

            .arg(Arg::with_name(ARG_DRM_I915)
                .long(ARG_DRM_I915)
                .takes_value(true)
                .value_name("IDS")
                .help("Target i915 card ids or pci ids, default all, ex. 0,1,3-5"))

            .arg(Arg::with_name(ARG_DRM_I915_MIN)
                .long(ARG_DRM_I915_MIN)
                .takes_value(true)
                .value_name("HERTZ")
                .help("Set i915 min frequency per --drm-i915, ex. 1200 or 1.2ghz"))

            .arg(Arg::with_name(ARG_DRM_I915_MAX)
                .long(ARG_DRM_I915_MAX)
                .takes_value(true)
                .value_name("HERTZ")
                .help("Set i915 max frequency per --drm-i915, ex. 1200 or 1.2ghz"))

            .arg(Arg::with_name(ARG_DRM_I915_BOOST)
                .long(ARG_DRM_I915_BOOST)
                .takes_value(true)
                .value_name("HERTZ")
                .help("Set i915 boost frequency per --drm-i915, ex. 1200 or 1.2ghz"));

        #[cfg(feature = "nvml")]
        let a = a

            .arg(Arg::with_name(ARG_NVML)
                .long(ARG_NVML)
                .takes_value(true)
                .value_name("IDS")
                .help("Target nvidia card ids or pci ids, default all, ex. 0,1,3-5"))

            .arg(Arg::with_name(ARG_NVML_GPU_MIN)
                .long(ARG_NVML_GPU_MIN)
                .takes_value(true)
                .value_name("HERTZ")
                .help("Set nvidia gpu min frequency per --nvml, ex. 1200 or 1.2ghz")
                .requires(ARG_NVML_GPU_MAX))

            .arg(Arg::with_name(ARG_NVML_GPU_MAX)
                .long(ARG_NVML_GPU_MAX)
                .takes_value(true)
                .value_name("HERTZ")
                .help("Set nvidia gpu max frequency per --nvml, ex. 1200 or 1.2ghz")
                .requires(ARG_NVML_GPU_MIN))

            .arg(Arg::with_name(ARG_NVML_GPU_RESET)
                .long(ARG_NVML_GPU_RESET)
                .takes_value(false)
                .conflicts_with("nvml-gpu-freq")
                .help("Reset nvidia gpu frequency to default per --nvml"))

            .arg(Arg::with_name(ARG_NVML_POWER_LIMIT)
                .long(ARG_NVML_POWER_LIMIT)
                .takes_value(true)
                .value_name("WATTS")
                .help("Set nvidia card power limit per --nvml"));

        let a = a

            .arg(Arg::with_name(ARG_PSTATE_EPB)
                .long(ARG_PSTATE_EPB)
                .takes_value(true)
                .value_name("0-15")
                .help("Set intel-pstate energy/performance bias per --cpu"))

            .arg(Arg::with_name(ARG_PSTATE_EPP)
                .long(ARG_PSTATE_EPP)
                .takes_value(true)
                .value_name("STR")
                .help("Set intel-pstate energy/performance pref per --cpu"))

            .arg(Arg::with_name(ARG_RAPL_PACKAGE)
                .short("P")
                .long(ARG_RAPL_PACKAGE)
                .takes_value(true)
                .value_name("INT")
                .help("Target intel-rapl package"))

            .arg(Arg::with_name(ARG_RAPL_ZONE)
                .short("Z")
                .long(ARG_RAPL_ZONE)
                .takes_value(true)
                .value_name("INT")
                .help("Target intel-rapl sub-zone"))

            .arg(Arg::with_name(ARG_RAPL_LONG_LIMIT)
                .short("L")
                .long(ARG_RAPL_LONG_LIMIT)
                .takes_value(true)
                .value_name("WATTS")
                .help("Set intel-rapl long_term power limit per --rapl-package/zone")
                .requires(ARG_RAPL_PACKAGE))

            .arg(Arg::with_name(ARG_RAPL_LONG_WINDOW)
                .long(ARG_RAPL_LONG_WINDOW)
                .takes_value(true)
                .value_name("SECS")
                .help("Set intel-rapl long_term time window per --rapl-package/zone")
                .requires(ARG_RAPL_PACKAGE))

            .arg(Arg::with_name(ARG_RAPL_SHORT_LIMIT)
                .short("S")
                .long(ARG_RAPL_SHORT_LIMIT)
                .takes_value(true)
                .value_name("WATTS")
                .help("Set intel-rapl short_term power limit per --rapl-package/zone")
                .requires(ARG_RAPL_PACKAGE))

            .arg(Arg::with_name(ARG_RAPL_SHORT_WINDOW)
                .long(ARG_RAPL_SHORT_WINDOW)
                .takes_value(true)
                .value_name("SECS")
                .help("Set intel-rapl short_term time window per --rapl-package/zone")
                .requires(ARG_RAPL_PACKAGE))

            .arg(Arg::with_name("CHAIN")
                .raw(true));

        a
    }

    // Parse and return the knobs call chain.
    fn chain(m: clap::ArgMatches) -> Result<crate::Chain> {
        let mut chain: Vec<crate::Knobs> = vec![];
        let mut argv: Vec<String>;
        let mut m = m;
        loop {
            chain.push(m.clone().try_into()?);
            if !m.is_present("CHAIN") { break; }
            m = match m.values_of("CHAIN") {
                Some(c) => {
                    let chain_args: Vec<String> = c.map(String::from).collect();
                    if chain_args.is_empty() { break; }
                    argv = vec![NAME.to_string()];
                    argv.extend(chain_args.into_iter());
                    Self::app(NAME).get_matches_from_safe(&argv)?
                },
                None => break,
            }
        };
        Ok(chain.into())
    }

    // Resolve resource ids. Some flags, e.g. --cpu, --nvml, accept a list of
    // resource ids, and will default to all resource ids when omitted. In the
    // latter case, this function will fill in the default resource ids as
    // required.
    async fn resolve(mut chain: crate::Chain) -> crate::Chain {
        for k in chain.iter_mut() {
            if k.has_cpu_related_values() && k.cpu.is_none() {
                k.cpu = CpuIdsOnce::get().await;
            }
            if k.has_drm_i915_values() && k.drm_i915.is_none() {
                k.drm_i915 = DrmI915IdsOnce::get().await
                    .map(|ids| ids
                        .into_iter()
                        .map(crate::CardId::Id)
                        .collect());
            }
            #[cfg(feature = "nvml")]
            if k.has_nvml_values() && k.nvml.is_none() {
                k.nvml = NvmlIdsOnce::get().await
                    .map(|ids| ids
                        .into_iter()
                        .map(crate::CardId::Id)
                        .collect());
            }
        }
        chain
    }

    // Determine the binary name from argv[0].
    fn argv0(argv: &[String]) -> &str {
        argv
            .first()
            .and_then(|s| s.as_str().split('/').last())
            .unwrap_or(NAME)
    }

    // Create new instance from command-line arguments.
    pub async fn from_args(argv: &[String]) -> Result<Self> {
        let a = Self::app(Self::argv0(argv));
        let m = a.get_matches_from_safe(argv)?;
        let s = Self {
            quiet: flag(ARG_QUIET, &m),
            show_cpu: flag(ARG_SHOW_CPU, &m),
            show_drm: flag(ARG_SHOW_DRM, &m),
            #[cfg(feature = "nvml")]
            show_nvml: flag(ARG_SHOW_NVML, &m),
            show_pstate: flag(ARG_SHOW_PSTATE, &m),
            show_rapl: flag(ARG_SHOW_RAPL, &m),
            chain: Self::resolve(Self::chain(m)?).await,
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

    async fn format_values(&self, s: &mut Vec<u8>, rapl_samplers: Option<RaplSamplers>) -> Result<()> {
        use crate::format::FormatValues as _;

        let show_all = !self.has_show_args();

        if show_all || self.show_cpu.is_some() {
            if let Some(cpu) = sysfs::cpu::Cpu::read(()).await {
                if let Some(cpufreq) = sysfs::cpufreq::Cpufreq::read(()).await {
                    (cpu, cpufreq).format_values(s, ()).await?;
                }
            }
        }

        if show_all || self.show_pstate.is_some() {
            if let Some(intel_pstate) = sysfs::intel_pstate::IntelPstate::read(()).await {
                intel_pstate.format_values(s, ()).await?;
            }
        }

        if show_all || self.show_rapl.is_some() {
            if let Some(intel_rapl) = sysfs::intel_rapl::IntelRapl::read(()).await {
                intel_rapl.format_values(s, rapl_samplers.clone()).await?;
            }
        }

        if show_all || self.show_drm.is_some() {
            if let Some(drm) = sysfs::drm::Drm::read(()).await {
                drm.format_values(s, ()).await?;
            }
        }

        #[cfg(feature = "nvml")]
        if show_all || self.show_nvml.is_some() {
            nvml_facade::Nvml.format_values(s, ()).await?;
        }

        Ok(())
    }

    const RAPL_RUNTIME: Duration = Duration::from_millis(400);
    const RAPL_INTERVAL: Duration = Duration::from_millis(100);
    const RAPL_COUNT: usize = 11;

    // Command-line interface app logic.
    pub async fn run(&self) -> Result<()> {
        use sysfs::tokio::Feature;
        let begin = CounterOnce::delta().await;
        let rapl_samplers =
            if self.quiet.is_none() &&
                (!self.has_show_args() || self.show_rapl.is_some()) &&
                sysfs::intel_rapl::IntelRapl::present().await
            {
                if let Some(s) = RaplSampler::all(Self::RAPL_INTERVAL, Self::RAPL_COUNT).await {
                    let mut s = RaplSamplers::from(s);
                    s.start().await;
                    Some(s)
                } else { None }
            } else { None };
        self.chain.apply_values().await;
        if self.quiet.is_some() { return Ok(()); }
        if let Some(samplers) = rapl_samplers.as_ref() {
            if samplers.working().await {
                let runtime = CounterOnce::delta().await - begin;
                if runtime < Self::RAPL_RUNTIME {
                    sleep(Self::RAPL_RUNTIME - runtime).await;
                }
            }
        }
        let mut s = Vec::with_capacity(3000);
        let format_res = self.format_values(&mut s, rapl_samplers.clone()).await;
        if let Some(mut samplers) = rapl_samplers { samplers.stop().await; }
        format_res?;
        println!("{}", String::from_utf8_lossy(&s).trim_end());
        Ok(())
    }
}

// Cli application.
#[derive(Clone, Debug)]
pub struct App;

impl App {

    // Run app with args.
    pub async fn run_with_args(args: &[String]) -> Result<()> {
        LoggingOnce::configure().await;
        match Cli::from_args(args).await {
            Ok(cli) => cli.run().await,
            Err(err) => {
                if let Error::Clap(e) = &err {
                    if let clap::ErrorKind::HelpDisplayed = e.kind {
                        println!("{}", e.message);
                        std::process::exit(0);
                    }
                }
                eprintln!("{}", err);
                std::process::exit(1);
            },
        }
    }

    // Run app.
    pub async fn run() -> Result<()> {
        let args: Vec<String> = std::env::args().collect();
        Self::run_with_args(&args).await
    }
}

// Configure logging env vars and default configuration.
#[derive(Clone, Debug)]
struct LoggingOnce;

impl LoggingOnce {
    pub async fn configure() {
        static LOGGING: OnceCell<()> = OnceCell::const_new();
        async fn init() {
            use std::io::Write;
            use env_logger::{Builder, Env};
            let env = Env::default()
                .filter_or("KNOBS_LOG", "error")
                .write_style_or("KNOBS_LOG_STYLE", "never");
            Builder::from_env(env)
                .format(|buf, record| {
                    writeln!(buf, "{}", record.args())
                })
                .init()
        }
        LOGGING.get_or_init(init).await;
    }
}

// Once of the `Instant` it was initialized.
#[derive(Clone, Debug)]
struct CounterOnce;

impl CounterOnce {
    pub async fn get() -> Instant {
        static START: OnceCell<Instant> = OnceCell::const_new();
        async fn make() -> Instant { Instant::now() }
        *START.get_or_init(make).await
    }

    pub async fn delta() -> Duration {
        let then = Self::get().await;
        Instant::now() - then
    }
}

// Once of cpu ids.
#[derive(Clone, Debug)]
struct CpuIdsOnce;

impl CpuIdsOnce {
    pub async fn get() -> Option<Vec<u64>> {
        static CPU_IDS: OnceCell<Option<Vec<u64>>> = OnceCell::const_new();
        async fn ids() -> Option<Vec<u64>> { sysfs::cpu::Policy::ids().await }
        CPU_IDS.get_or_init(ids).await.clone()
    }
}

// Once of drm card ids.
#[derive(Clone, Debug)]
struct DrmIdsOnce;

impl DrmIdsOnce {
    pub async fn get() -> Option<Vec<u64>> {
        static DRM_IDS: OnceCell<Option<Vec<u64>>> = OnceCell::const_new();
        async fn ids() -> Option<Vec<u64>> { sysfs::drm::Card::ids().await }
        DRM_IDS.get_or_init(ids).await.clone()
    }
}

// Once of i915 card ids.
#[derive(Clone, Debug)]
struct DrmI915IdsOnce;

impl DrmI915IdsOnce {
    pub async fn get() -> Option<Vec<u64>> {
        use sysfs::drm::io::tokio::driver;
        static DRM_I915_IDS: OnceCell<Option<Vec<u64>>> = OnceCell::const_new();
        async fn ids() -> Option<Vec<u64>> {
            let mut ids = vec![];
            if let Some(drm_ids) = DrmIdsOnce::get().await {
                for id in drm_ids {
                    if let Ok("i915") = driver(id).await.as_deref() {
                        ids.push(id);
                    }
                }
            }
            if ids.is_empty() { None } else { Some(ids) }
        }
        DRM_I915_IDS.get_or_init(ids).await.clone()
    }
}

// Once of nvml card ids.
#[cfg(feature = "nvml")]
#[derive(Clone, Debug)]
struct NvmlIdsOnce;

#[cfg(feature = "nvml")]
impl NvmlIdsOnce {
    pub async fn get() -> Option<Vec<u64>> {
        static NVML_IDS: OnceCell<Option<Vec<u64>>> = OnceCell::const_new();
        async fn ids() -> Option<Vec<u64>> {
            nvml_facade::Nvml::ids()
                .map(|ids| ids.into_iter().map(u64::from).collect())
        }
        NVML_IDS.get_or_init(ids).await.clone()
    }
}
