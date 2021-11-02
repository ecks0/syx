use zysfs::types::{self as sysfs, tokio::Read as _};
use std::{collections::HashMap, convert::TryFrom, path::PathBuf, str::FromStr, time::{Duration, Instant}};
use tokio::{sync::OnceCell, time::sleep};
use crate::{Chain, Error, Knobs, Result, data::{RaplSampler, RaplSamplers}};

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

const ARG_PROFILE: &str           = "PROFILE";

const ARG_CHAIN: &str             = "CHAIN";

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

// Build and return a clap app.
fn app<'a, 'b>() -> clap::App<'a, 'b> {
    use clap::{App, AppSettings, Arg, crate_version};

    let a = App::new(NAME)

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

        .arg(Arg::with_name(ARG_PROFILE))

        .arg(Arg::with_name(ARG_CHAIN)
            .raw(true));

    a
}

// Load argument values from env vars.
#[derive(Debug)]
struct Env;

impl Env {

    pub fn hostname() -> Option<String> {
        let mut buf = [0u8; 64];
        match nix::unistd::gethostname(&mut buf) {
            Ok(h) => match h.to_str() {
                Ok(h) => Some(h.to_string()),
                Err(_) => {
                    log::error!("ERR nix r UTF8 hostname() utf8 conversion failed");
                    None
                }
            },
            Err(e) => {
                log::error!("ERR nix r {} hostname() failed to determine system hostname", e);
                None
            },
        }
    }

    // Return the environment variable value, if any, for the given cli flag name.
    pub fn var(cli_name: &str) -> Option<String> {

        fn var_name(cli_name: &str) -> String {
            format!("KNOBS_{}", cli_name.to_uppercase().replace("-", "_"))
        }

        match std::env::var(&var_name(cli_name)) {
            Ok(v) => {
                log::debug!("--{}: using value from environment: {}", cli_name, v);
                Some(v)
            },
            _ => None,
        }
    }
}

// Argument parsing helper.
#[derive(Clone, Debug)]
struct Parser<'a>(clap::ArgMatches<'a>);

impl<'a> Parser<'a> {

    pub fn new(argv: &[String]) -> Result<Self> {
        let m = app().get_matches_from_safe(argv)?;
        Ok(Self(m))
    }

    // Return true if the given argument is present in argv (not env vars).
    pub fn arg_present(&self, name: &str) -> bool { self.0.is_present(name) }

    // Return the values for an argument from argv (not env vars).
    pub fn arg_values(&self, name: &str) -> Option<clap::Values> { self.0.values_of(name) }

    // Parse a flag argument from the argv or from env vars if present.
    pub fn flag(&self, name: &str) -> Option<()> {
        match self.0.is_present(name) {
            true => Some(()),
            false =>
                match Env::var(name)
                    .map(|v| !v.is_empty() && v != "0" && v.to_lowercase() != "false")
                    .unwrap_or(false)
                {
                    true => Some(()),
                    false => None,
                },
        }
    }

    // Parse an integer argument from the argv or from env vars.
    pub fn int<T: FromStr<Err = std::num::ParseIntError>>(&self, name: &str) -> Result<Option<T>> {
        match self.0.value_of(name)
            .map(|v| v.to_string())
            .or_else(|| Env::var(name))
        {
            Some(v) => Ok(Some(
                T::from_str(&v)
                    .map_err(|_| Error::parse_flag(name, "Expected integer value".into()))?
            )),
            None => Ok(None),
        }
    }

    // Parse a string argument from the argv or from env vars.
    pub fn str(&self, name: &str) -> Option<String> {
        self.0.value_of(name)
            .map(|v| v.to_string())
            .or_else(|| Env::var(name))
    }

    // Parse an argument using `FromStr` from the argv or from env vars.
    pub fn from_str<S>(&self, name: &str) -> Result<Option<S>>
    where
        S: FromStr<Err = Error>,
    {
        match self.0.value_of(name)
            .map(String::from)
            .or_else(|| Env::var(name))
        {
            Some(v) => Ok(Some(
                S::from_str(&v)
                    .map_err(|e| Error::parse_flag(name, e.to_string()))?
            )),
            None => Ok(None),
        }
    }

    // Parse an argument using `FromStr` from the argv or from env vars
    // and convert to the given type.
    pub fn from_str_as<S, T>(&self, name: &str) -> Result<Option<T>>
    where
        S: FromStr<Err = Error>,
        T: From<S>,
    {
        Ok(self.from_str::<S>(name)?.map(|v| T::from(v)))
    }
}

impl<'a> TryFrom<Parser<'a>> for Knobs {
    type Error = Error;

    fn try_from(p: Parser<'a>) -> Result<Self> {
        let s = Self {
            cpu: p.from_str_as::<crate::Indices, _>(ARG_CPU)?,
            cpu_online: p.from_str_as::<crate::BoolStr, _>(ARG_CPU_ONLINE)?,
            cpufreq_gov: p.str(ARG_CPUFREQ_GOV),
            cpufreq_min: p.from_str_as::<crate::FrequencyStr, _>(ARG_CPUFREQ_MIN)?,
            cpufreq_max: p.from_str_as::<crate::FrequencyStr, _>(ARG_CPUFREQ_MAX)?,
            drm_i915: p.from_str_as::<crate::CardIds, _>(ARG_DRM_I915)?,
            drm_i915_min: p.from_str_as::<crate::FrequencyStr, _>(ARG_DRM_I915_MIN)?,
            drm_i915_max: p.from_str_as::<crate::FrequencyStr, _>(ARG_DRM_I915_MAX)?,
            drm_i915_boost: p.from_str_as::<crate::FrequencyStr, _>(ARG_DRM_I915_BOOST)?,
            #[cfg(feature = "nvml")]
            nvml: p.from_str_as::<crate::CardIds, _>(ARG_NVML)?,
            #[cfg(feature = "nvml")]
            nvml_gpu_min: p.from_str_as::<crate::FrequencyStr, _>(ARG_NVML_GPU_MIN)?,
            #[cfg(feature = "nvml")]
            nvml_gpu_max: p.from_str_as::<crate::FrequencyStr, _>(ARG_NVML_GPU_MAX)?,
            #[cfg(feature = "nvml")]
            nvml_gpu_reset: p.flag(ARG_NVML_GPU_RESET).map(|_| true),
            #[cfg(feature = "nvml")]
            nvml_power_limit: p.from_str_as::<crate::PowerStr, _>(ARG_NVML_POWER_LIMIT)?,
            pstate_epb: p.int::<u64>(ARG_PSTATE_EPB)?,
            pstate_epp: p.str(ARG_PSTATE_EPP),
            rapl_package: p.int::<u64>(ARG_RAPL_PACKAGE)?,
            rapl_zone: p.int::<u64>(ARG_RAPL_ZONE)?,
            rapl_long_limit: p.from_str_as::<crate::PowerStr, _>(ARG_RAPL_LONG_LIMIT)?,
            rapl_long_window: p.from_str_as::<crate::DurationStr, _>(ARG_RAPL_LONG_WINDOW)?,
            rapl_short_limit: p.from_str_as::<crate::PowerStr, _>(ARG_RAPL_SHORT_LIMIT)?,
            rapl_short_window: p.from_str_as::<crate::DurationStr, _>(ARG_RAPL_SHORT_WINDOW)?,
        };
        Ok(s)
   }
}

// Builders for important paths.
#[derive(Debug)]
struct Paths;

impl Paths {

    // Sub-directory containing profile files.
    const PROFILE: &'static str = "profile";

    // Name of state file.
    // const STATE: &'static str = "state.yaml";

    // e.g. ~/.config/knobs
    fn config_user() -> Option<PathBuf> {
        dirs::config_dir()
            .map(|mut p| {
                p.push(NAME);
                p
            })
    }

    // /etc/knobs
    fn config_sys() -> PathBuf {
        let mut p = PathBuf::new();
        p.push("/etc");
        p.push(NAME);
        p
    }

    // e.g. ~/.config/knobs/profile/<file_name>
    pub fn profile_user(file_name: &str) -> Option<PathBuf> {
        Self::config_user()
            .map(|mut p| {
                p.push(Self::PROFILE);
                p.push(file_name);
                p
            })
    }

    // /etc/knobs/profile/<file_name>
    pub fn profile_sys(file_name: &str) -> PathBuf {
        let mut p = Self::config_sys();
        p.push(Self::PROFILE);
        p.push(file_name);
        p
    }

    // e.g. ~/.local/state/knobs/state.yaml
    // pub fn state() -> Option<PathBuf> {
    //     dirs::state_dir()
    //         .map(|mut p| {
    //             p.push(NAME);
    //             p.push(Self::STATE);
    //             p
    //         })
    // }
}

// Profile loader.
#[derive(Debug)]
struct Profile;

impl Profile {

    // Return a list of possible paths for the profile file.
    fn search_paths() -> Vec<PathBuf> {
        let mut paths = vec![];
        for base_name in [Env::hostname().as_deref(), Some(NAME)].into_iter().flatten() {
            let file_name = format!("{}.yaml", base_name);
            if let Some(p) = Paths::profile_user(&file_name) { paths.push(p); }
            let p = Paths::profile_sys(&file_name);
            paths.push(p);
        }
        paths
    }

    // Return the path to the profile file.
    fn path() -> Option<PathBuf> {
        Self::search_paths()
            .into_iter()
            .find(|p| p.is_file())
    }

    // Load the given profile name from the profile file.
    pub async fn get(name: &str) -> Option<Chain> {
        let path =
            if let Some(p) = Self::path() { p } else {
                use std::io::Write;
                let search_paths = Self::search_paths();
                let mut v = vec![];
                writeln!(v, "Error: Profile file not found at...").unwrap();
                for p in search_paths { writeln!(v, "  {}", p.display()).unwrap(); }
                log::error!("{}", String::from_utf8_lossy(&v).trim_end());
                return None;
            };
        match tokio::fs::read_to_string(&path).await {
            Ok(s) => match serde_yaml::from_str::<HashMap<String, Chain>>(&s) {
                Ok(p) => match p.into_iter().find(|(n, _)| n == name).map(|(_, c)| c) {
                    Some(c) => Some(c),
                    None => {
                        log::error!("Error: profile '{}' not found in {}", name, path.display());
                        None
                    }
                },
                Err(e) => {
                    log::error!("Error: {}: {}", e, path.display());
                    None
                },
            },
            Err(e) => {
                log::error!("Error: {}: {}", e, path.display());
                None
            },
        }
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
    pub profile: Option<String>,
    pub chain: Chain,
}

impl Cli {
    // Parse and return the knobs call chain.
    fn chain(first: Parser) -> Result<Chain> {
        let mut chain: Vec<Knobs> = vec![];
        let mut p = first;
        loop {
            chain.push(Knobs::try_from(p.clone())?);
            if !p.arg_present(ARG_CHAIN) { break; }
            match p.arg_values(ARG_CHAIN) {
                Some(v) => {
                    let mut v: Vec<String> = v.map(String::from).collect();
                    if v.is_empty() { break; }
                    v.insert(0, NAME.to_string());
                    p = Parser::new(&v)?;
                },
                None => break,
            };
        }
        Ok(chain.into())
    }

    // Resolve resource ids. Some flags, e.g. --cpu, --nvml, accept a list of
    // resource ids, and will default to all resource ids when omitted. In the
    // latter case, this function will fill in the default resource ids as
    // required.
    async fn resolve(mut chain: Chain) -> Chain {
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

    pub async fn new(argv: &[String]) -> Result<Self> {
        let p = Parser::new(argv)?;
        let s = Self {
            quiet: p.flag(ARG_QUIET),
            show_cpu: p.flag(ARG_SHOW_CPU),
            show_drm: p.flag(ARG_SHOW_DRM),
            #[cfg(feature = "nvml")]
            show_nvml: p.flag(ARG_SHOW_NVML),
            show_pstate: p.flag(ARG_SHOW_PSTATE),
            show_rapl: p.flag(ARG_SHOW_RAPL),
            profile: p.str(ARG_PROFILE),
            chain: Self::resolve(Self::chain(p)?).await,
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
    async fn print_values(&self, b: &mut Vec<u8>, rapl_samplers: Option<RaplSamplers>) -> Result<()> {
        use crate::format::FormatValues as _;
        let show_all = !self.has_show_args();
        if show_all || self.show_cpu.is_some() {
            if let Some(cpu) = sysfs::cpu::Cpu::read(()).await {
                if let Some(cpufreq) = sysfs::cpufreq::Cpufreq::read(()).await {
                    (cpu, cpufreq).format_values(b, ()).await?;
                }
            }
        }
        if show_all || self.show_pstate.is_some() {
            if let Some(intel_pstate) = sysfs::intel_pstate::IntelPstate::read(()).await {
                intel_pstate.format_values(b, ()).await?;
            }
        }
        if show_all || self.show_rapl.is_some() {
            if let Some(intel_rapl) = sysfs::intel_rapl::IntelRapl::read(()).await {
                intel_rapl.format_values(b, rapl_samplers.clone()).await?;
            }
        }
        if show_all || self.show_drm.is_some() {
            if let Some(drm) = sysfs::drm::Drm::read(()).await {
                drm.format_values(b, ()).await?;
            }
        }
        #[cfg(feature = "nvml")]
        if show_all || self.show_nvml.is_some() {
            nvml_facade::Nvml.format_values(b, ()).await?;
        }
        Ok(())
    }

    // Minimum run time required for rapl samplers to give useful data.
    const RAPL_RUNTIME: Duration = Duration::from_millis(400);

    // Sleep time between samples, per sampler/zone.
    const RAPL_INTERVAL: Duration = Duration::from_millis(100);

    /// Maximum number of samples, per sampler/zone.
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
        if let Some(profile) = self.profile.as_ref() {
            match Profile::get(profile).await {
                Some(chain) => chain.apply_values().await,
                None => std::process::exit(1),
            }
        }
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
        let format_res = self.print_values(&mut s, rapl_samplers.clone()).await;
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
        match Cli::new(args).await {
            Ok(cli) => cli.run().await,
            Err(err) => {
                if let Error::Clap(e) = &err {
                    if let clap::ErrorKind::HelpDisplayed = e.kind {
                        println!("{}", e.message);
                        std::process::exit(0);
                    }
                }
                log::error!("{}", err);
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
#[derive(Debug)]
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
#[derive(Debug)]
struct CounterOnce;

impl CounterOnce {
    pub async fn get() -> Instant {
        static START: OnceCell<Instant> = OnceCell::const_new();
        async fn start() -> Instant { Instant::now() }
        *START.get_or_init(start).await
    }

    pub async fn delta() -> Duration {
        let then = Self::get().await;
        Instant::now() - then
    }
}

// Once of cpu ids.
#[derive(Debug)]
struct CpuIdsOnce;

impl CpuIdsOnce {
    pub async fn get() -> Option<Vec<u64>> {
        static CPU_IDS: OnceCell<Option<Vec<u64>>> = OnceCell::const_new();
        async fn cpu_ids() -> Option<Vec<u64>> { sysfs::cpu::Policy::ids().await }
        CPU_IDS.get_or_init(cpu_ids).await.clone()
    }
}

// Once of drm card ids.
#[derive(Debug)]
struct DrmIdsOnce;

impl DrmIdsOnce {
    pub async fn get() -> Option<Vec<u64>> {
        static DRM_IDS: OnceCell<Option<Vec<u64>>> = OnceCell::const_new();
        async fn drm_ids() -> Option<Vec<u64>> { sysfs::drm::Card::ids().await }
        DRM_IDS.get_or_init(drm_ids).await.clone()
    }
}

// Once of i915 card ids.
#[derive(Debug)]
struct DrmI915IdsOnce;

impl DrmI915IdsOnce {
    pub async fn get() -> Option<Vec<u64>> {
        use sysfs::drm::io::tokio::driver;
        static DRM_I915_IDS: OnceCell<Option<Vec<u64>>> = OnceCell::const_new();
        async fn drm_i915_ids() -> Option<Vec<u64>> {
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
        DRM_I915_IDS.get_or_init(drm_i915_ids).await.clone()
    }
}

// Once of nvml card ids.
#[cfg(feature = "nvml")]
#[derive(Debug)]
struct NvmlIdsOnce;

#[cfg(feature = "nvml")]
impl NvmlIdsOnce {
    pub async fn get() -> Option<Vec<u64>> {
        static NVML_IDS: OnceCell<Option<Vec<u64>>> = OnceCell::const_new();
        async fn nvml_ids() -> Option<Vec<u64>> {
            nvml_facade::Nvml::ids()
                .map(|ids| ids.into_iter().map(u64::from).collect())
        }
        NVML_IDS.get_or_init(nvml_ids).await.clone()
    }
}
