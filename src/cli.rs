use log::{Level, debug, info, log_enabled, trace};
use zysfs::types::{self as sysfs};
use std::{convert::TryFrom, str::FromStr};
use tokio::io::AsyncWriteExt;
use crate::{Error, Result};

const AFTER_HELP: &str = r#"             IDS   A comma-delimited sequence of integers and/or integer ranges.
         TOGGLES   An sequence of 0 (off), 1 (on) or _ (skip) characters.
              HZ*  mhz when unspecified: hz/h - khz/k - mhz/m - ghz/g - thz/t
           WATTS*  mw when unspecified: uw/u - mw/m - w - kw/k
            SECS   ms when unspecified: ns/n - us/u - ms/m - s

        * Floating point values may be given for these units.

    Values for supported hardware are shown unless the --show-* or --quiet flags are used.

    All flags may be expressed as env vars. For example:

        --show-cpu                 → KNOBS_SHOW_CPU=1
        --cpu 1,3-5                → KNOBS_CPU=1,3-5
        --drm-i915-boost 1.2ghz    → KNOBS_DRM_I915_BOOST=1.2ghz

    The KNOBS_LOG env var may be set to trace, debug, info, warn, or error (default).
"#;

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

        .arg(Arg::with_name("quiet")
            .short("q")
            .long("quiet")
            .takes_value(false)
            .help("Do not print values"))

        .arg(Arg::with_name("show-cpu")
            .long("show-cpu")
            .takes_value(false)
            .help("Print cpu and cpufreq values"))

        .arg(Arg::with_name("show-drm")
            .long("show-drm")
            .takes_value(false)
            .help("Print drm values"));

    #[cfg(feature = "nvml")]
    let a = a

        .arg(Arg::with_name("show-nvml")
            .long("show-nvml")
            .takes_value(false)
            .help("Print nvidia management values"));

    let a = a

        .arg(Arg::with_name("show-pstate")
            .long("show-pstate")
            .takes_value(false)
            .help("Print intel-pstate values"))

        .arg(Arg::with_name("show-rapl")
            .long("show-rapl")
            .takes_value(false)
            .help("Print intel-rapl values"))

        .arg(Arg::with_name("cpu")
            .short("c")
            .long("cpu")
            .takes_value(true)
            .value_name("IDS")
            .help("Target cpu ids, default all, ex. 0,1,3-5"))

        .arg(Arg::with_name("cpu-on")
            .short("o")
            .long("cpu-on")
            .takes_value(true)
            .value_name("0|1")
            .help("Set cpu online status per --cpu"))

        .arg(Arg::with_name("cpus-on")
            .short("O")
            .long("cpus-on")
            .takes_value(true)
            .value_name("TOGGLES")
            .help("Set cpu online status, ex. 10_1 → 0=ON 1=OFF 2=SKIP 3=ON"))

        .arg(Arg::with_name("cpufreq-gov")
            .short("g")
            .long("cpufreq-gov")
            .takes_value(true)
            .value_name("STR")
            .help("Set cpufreq governor per --cpu"))

        .arg(Arg::with_name("cpufreq-min")
            .short("n")
            .long("cpufreq-min")
            .takes_value(true)
            .value_name("HZ")
            .help("Set cpufreq min freq per --cpu, ex. 1200 or 1.2ghz"))

        .arg(Arg::with_name("cpufreq-max")
            .short("x")
            .long("cpufreq-max")
            .takes_value(true)
            .value_name("HZ")
            .help("Set cpufreq max freq per --cpu, ex. 1200 or 1.2ghz"))

        .arg(Arg::with_name("drm-i915")
            .long("drm-i915")
            .takes_value(true)
            .value_name("IDS")
            .help("Target i915 card ids or pci ids, default all, ex. 0,1,3-5"))

        .arg(Arg::with_name("drm-i915-min")
            .long("drm-i915-min")
            .takes_value(true)
            .value_name("HZ")
            .help("Set i915 min frequency per --drm-i915, ex. 1200 or 1.2ghz"))

        .arg(Arg::with_name("drm-i915-max")
            .long("drm-i915-max")
            .takes_value(true)
            .value_name("HZ")
            .help("Set i915 max frequency per --drm-i915, ex. 1200 or 1.2ghz"))

        .arg(Arg::with_name("drm-i915-boost")
            .long("drm-i915-boost")
            .takes_value(true)
            .value_name("HZ")
            .help("Set i915 boost frequency per --drm-i915, ex. 1200 or 1.2ghz"));

    #[cfg(feature = "nvml")]
    let a = a

        .arg(Arg::with_name("nvml")
            .long("nvml")
            .takes_value(true)
            .value_name("IDS")
            .help("Target nvidia card ids or pci ids, default all, ex. 0,1,3-5"))

        .arg(Arg::with_name("nvml-gpu-min")
            .long("nvml-gpu-min")
            .takes_value(true)
            .value_name("HZ")
            .requires("nvml-gpu-max")
            .help("Set nvidia gpu min frequency per --nvml, ex. 1200 or 1.2ghz"))

        .arg(Arg::with_name("nvml-gpu-max")
            .long("nvml-gpu-max")
            .takes_value(true)
            .value_name("HZ")
            .requires("nvml-gpu-min")
            .help("Set nvidia gpu max frequency per --nvml, ex. 1200 or 1.2ghz"))

        .arg(Arg::with_name("nvml-gpu-reset")
            .long("nvml-gpu-reset")
            .takes_value(false)
            .conflicts_with("nvml-gpu-freq")
            .help("Reset nvidia gpu frequency to default per --nvml"))

        .arg(Arg::with_name("nvml-power-limit")
            .long("nvml-power-limit")
            .takes_value(true)
            .value_name("WATTS")
            .help("Set nvidia card power limit per --nvml"));

    let a = a

        .arg(Arg::with_name("pstate-epb")
            .long("pstate-epb")
            .takes_value(true)
            .value_name("0-15")
            .help("Set intel-pstate energy/performance bias per --cpu"))

        .arg(Arg::with_name("pstate-epp")
            .long("pstate-epp")
            .takes_value(true)
            .value_name("STR")
            .help("Set intel-pstate energy/performance pref per --cpu"))

        .arg(Arg::with_name("rapl-package")
            .short("P")
            .long("rapl-package")
            .takes_value(true)
            .value_name("INT")
            .help("Target intel-rapl package, default 0"))

        .arg(Arg::with_name("rapl-zone")
            .short("Z")
            .long("rapl-zone")
            .takes_value(true)
            .value_name("INT")
            .help("Target intel-rapl sub-zone, default none"))

        .arg(Arg::with_name("rapl-c0-limit")
            .short("0")
            .long("rapl-c0-limit")
            .takes_value(true)
            .value_name("WATTS")
            .help("Set intel-rapl c0 power limit per --rapl-{package,zone}"))

        .arg(Arg::with_name("rapl-c1-limit")
            .short("1")
            .long("rapl-c1-limit")
            .takes_value(true)
            .value_name("WATTS")
            .help("Set intel-rapl c1 power limit per --rapl-{package,zone}"))

        .arg(Arg::with_name("rapl-c0-window")
            .long("rapl-c0-window")
            .takes_value(true)
            .value_name("SECS")
            .help("Set intel-rapl c0 time window per --rapl-{package,zone}"))

        .arg(Arg::with_name("rapl-c1-window")
            .long("rapl-c1-winodw")
            .takes_value(true)
            .value_name("SECS")
            .help("Set intel-rapl c1 time window per --rapl-{package,zone}"))

        .arg(Arg::with_name("chain")
            .raw(true));

    a
}

// Build a clap error.
// fn clap_error(kind: clap::ErrorKind, message: String) -> clap::Error {
//     let info = None;
//     clap::Error { message, kind, info }
// }

// Convert a cli flag name to an environment variable name.
fn env_name(cli_name: &str) -> String {
    format!("KNOBS_{}", cli_name.to_uppercase().replace("-", "_"))
}

// Return the environment variable value, if any, for the given cli flag name.
fn env_value(cli_name: &str) -> Option<String> {
    match std::env::var(&env_name(cli_name)) {
        Ok(v) => {
            debug!("--{}: using value from environment: {}", cli_name, v);
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
                .map_err(|_| Error::parse_flag(name, "expected integer value".into()))?
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
            cpu: arg::<crate::Indices>("cpu", &m)?.map(|v| v.into()),
            cpu_on: arg::<crate::BoolStr>("cpu-on", &m)?.map(|v| v.into()),
            cpus_on: arg::<crate::Toggles>("cpus-on", &m)?.map(|v| v.into()),
            cpufreq_gov: arg_str("cpufreq-gov", &m),
            cpufreq_min: arg::<crate::FrequencyStr>("cpufreq-min", &m)?.map(|v| v.into()),
            cpufreq_max: arg::<crate::FrequencyStr>("cpufreq-max", &m)?.map(|v| v.into()),
            drm_i915: arg::<crate::CardIds>("drm-i915", &m)?.map(|v| v.into()),
            drm_i915_min: arg::<crate::FrequencyStr>("drm-i915-min", &m)?.map(|v| v.into()),
            drm_i915_max: arg::<crate::FrequencyStr>("drm-i915-max", &m)?.map(|v| v.into()),
            drm_i915_boost: arg::<crate::FrequencyStr>("drm-i915-boost", &m)?.map(|v| v.into()),
            #[cfg(feature = "nvml")] nvml: arg::<crate::CardIds>("nvml", &m)?.map(|v| v.into()),
            #[cfg(feature = "nvml")] nvml_gpu_min: arg::<crate::FrequencyStr>("nvml-gpu-min", &m)?.map(|v| v.into()),
            #[cfg(feature = "nvml")] nvml_gpu_max: arg::<crate::FrequencyStr>("nvml-gpu-max", &m)?.map(|v| v.into()),
            #[cfg(feature = "nvml")] nvml_gpu_reset: flag("nvml-gpu-reset", &m).map(|_| true),
            #[cfg(feature = "nvml")] nvml_power_limit: arg::<crate::PowerStr>("nvml-power-limit", &m)?.map(|v| v.into()),
            pstate_epb: arg_int::<u64>("pstate-epb", &m)?,
            pstate_epp: arg_str("pstate-epp", &m),
            rapl_package: arg_int::<u64>("rapl-package", &m)?.or(Some(0)),
            rapl_zone: arg_int::<u64>("rapl-zone", &m)?,
            rapl_c0_limit: arg::<crate::PowerStr>("rapl-c0-limit", &m)?.map(|v| v.into()),
            rapl_c1_limit: arg::<crate::PowerStr>("rapl-c1-limit", &m)?.map(|v| v.into()),
            rapl_c0_window: arg::<crate::DurationStr>("rapl-c0-window", &m)?.map(|v| v.into()),
            rapl_c1_window: arg::<crate::DurationStr>("rapl-c1-window", &m)?.map(|v| v.into()),
        };
        Ok(s)
   }
}

// Parse and return the knobs call chain.
fn chain(a: clap::App, m: clap::ArgMatches) -> Result<Vec<crate::Knobs>> {
    let mut argv: Vec<String>;
    let mut m = m;
    let mut chain: Vec<crate::Knobs> = vec![];
    loop {
        chain.push(m.clone().try_into()?);
        if !m.is_present("chain") { break; }
        m = match m.values_of("chain") {
            Some(c) => {
                let chain_args: Vec<String> = c.map(String::from).collect();
                if chain_args.is_empty() { break; }
                argv = vec!["knobs".to_string()];
                argv.extend(chain_args.into_iter());
                a.clone().get_matches_from_safe(&argv)?
            },
            None => break,
        }
    };
    Ok(chain)
}

// Fetch any resource ids that need resolving.
async fn resolve(mut chain: Vec<crate::Knobs>) -> Result<Vec<crate::Knobs>> {
    for k in chain.iter_mut() {
        if k.has_cpu_or_related_values() && k.cpu.is_none() {
            k.cpu = crate::policy::cpu_ids_cached().await;
        }
        if k.has_drm_i915_values() && k.drm_i915.is_none() {
            k.drm_i915 = crate::policy::drm_ids_i915_cached().await
                .map(|ids| ids
                    .into_iter()
                    .map(crate::CardId::Id)
                    .collect());
        }
        #[cfg(feature = "nvml")]
        if k.has_nvml_values() && k.nvml.is_none() {
            k.nvml = crate::policy::nvml_ids_cached().await
                .map(|ids| ids
                    .into_iter()
                    .map(crate::CardId::Id)
                    .collect());
        }
    }
    Ok(chain)
}

// Determine the binary name from argv[0].
fn argv0(argv: &[String]) -> &str {
    const DEFAULT: &str = "knobs";
    argv
        .first()
        .map(|s| s.as_str())
        .unwrap_or(DEFAULT)
        .split('/')
        .last()
        .unwrap_or(DEFAULT)
}

#[derive(Clone, Debug)]
pub struct Cli {
    pub quiet: Option<()>,
    pub show_cpu: Option<()>,
    pub show_drm: Option<()>,
    #[cfg(feature = "nvml")] pub show_nvml: Option<()>,
    pub show_pstate: Option<()>,
    pub show_rapl: Option<()>,
    pub chain: Vec<crate::Knobs>,
}

impl Cli {

    pub fn setup_logging() {
        use std::io::Write;

        use env_logger::{Builder, Env};
        let env = Env::default()
            .filter_or("KNOBS_LOG", "error")
            .write_style_or("KNOBS_LOG_STYLE", "never");
        Builder::from_env(env)
            .format(|buf, record| {
                writeln!(buf, "{}", record.args())
            })
            .init();
    }

    pub async fn parse(argv: &[String]) -> Result<Self> {
        let a = app(argv0(argv));
        let m = a.clone().get_matches_from_safe(argv)?;
        let s = Self {
            quiet: flag("quiet", &m),
            show_cpu: flag("show-cpu", &m),
            show_drm: flag("show-drm", &m),
            #[cfg(feature = "nvml")] show_nvml: flag("show-nvml", &m),
            show_pstate: flag("show-pstate", &m),
            show_rapl: flag("show-rapl", &m),
            chain: resolve(chain(a, m)?).await?,
        };
        Ok(s)
    }

    fn has_show_args(&self) -> bool {
        let b = self.show_cpu.is_some() ||
            self.show_drm.is_some() ||
            self.show_pstate.is_some() ||
            self.show_rapl.is_some();
        #[cfg(feature = "nvml")]
        let b = b || self.show_nvml.is_some();
        b
    }

    pub async fn run(&self) -> Result<()> {
        use sysfs::tokio::Read as _;
        use crate::format::Format as _;

        for (i, knobs) in self.chain.iter().enumerate() {
            info!("Chain {}", i);
            if log_enabled!(Level::Trace) { trace!("{:#?}", knobs); }
            knobs.apply().await;
        }
        if self.quiet.is_none() {
            let show_all = !self.has_show_args();
            let mut s = vec![];
            s.write_all("\n".as_bytes()).await?;
            if show_all || self.show_cpu.is_some() {
                if let Some(cpu) = sysfs::cpu::Cpu::read(()).await {
                    if let Some(cpufreq) = sysfs::cpufreq::Cpufreq::read(()).await {
                        (cpu, cpufreq).format_values(&mut s).await?;
                    }
                }
            }
            if show_all || self.show_pstate.is_some() {
                if let Some(intel_pstate) = sysfs::intel_pstate::IntelPstate::read(()).await {
                    intel_pstate.format_values(&mut s).await?;
                }
            }
            if show_all || self.show_rapl.is_some() {
                if let Some(intel_rapl) = sysfs::intel_rapl::IntelRapl::read(()).await {
                    intel_rapl.format_values(&mut s).await?;
                }
            }
            if show_all || self.show_drm.is_some() {
                if let Some(drm) = sysfs::drm::Drm::read(()).await {
                    drm.format_values(&mut s).await?;
                }
            }
            #[cfg(feature = "nvml")]
            if show_all || self.show_nvml.is_some() {
                use nvml_facade::Nvml;
                Nvml.format_values(&mut s).await?;
            }
            print!("{}", String::from_utf8_lossy(&s));
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct App;

impl App {
    pub async fn run() -> Result<()> {
        Cli::setup_logging();
        let args: Vec<String> = std::env::args().collect();
        match Cli::parse(&args).await {
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
}
