use std::str::FromStr;

use clap::{crate_version, App, AppSettings, Arg};

use crate::cli::group::{Group, Groups};
use crate::cli::parse::{BoolStr, CardIds, DurationStr, FrequencyStr, Indices, PowerStr, Toggles};
use crate::cli::{env, Error, Result, NAME};

pub(super) const ARG_QUIET: &str = "quiet";

pub(super) const ARG_SHOW_CPU: &str = "show-cpu";
pub(super) const ARG_SHOW_PSTATE: &str = "show-pstate";
pub(super) const ARG_SHOW_RAPL: &str = "show-rapl";
pub(super) const ARG_SHOW_I915: &str = "show-i915";
#[cfg(feature = "nvml")]
pub(super) const ARG_SHOW_NV: &str = "show-nv";

pub(super) const ARG_CPU: &str = "cpu";
pub(super) const ARG_CPU_ON: &str = "cpu-on";
pub(super) const ARG_CPU_ON_EACH: &str = "cpu-on-each";

pub(super) const ARG_CPUFREQ_GOV: &str = "cpufreq-gov";
pub(super) const ARG_CPUFREQ_MIN: &str = "cpufreq-min";
pub(super) const ARG_CPUFREQ_MAX: &str = "cpufreq-max";

pub(super) const ARG_PSTATE_EPB: &str = "pstate-epb";
pub(super) const ARG_PSTATE_EPP: &str = "pstate-epp";

pub(super) const ARG_RAPL_PACKAGE: &str = "rapl-package";
pub(super) const ARG_RAPL_ZONE: &str = "rapl-zone";
pub(super) const ARG_RAPL_LONG_LIMIT: &str = "rapl-long-limit";
pub(super) const ARG_RAPL_LONG_WINDOW: &str = "rapl-long-window";
pub(super) const ARG_RAPL_SHORT_LIMIT: &str = "rapl-short-limit";
pub(super) const ARG_RAPL_SHORT_WINDOW: &str = "rapl-short-window";

pub(super) const ARG_I915: &str = "i915";
pub(super) const ARG_I915_MIN: &str = "i915-min";
pub(super) const ARG_I915_MAX: &str = "i915-max";
pub(super) const ARG_I915_BOOST: &str = "i915-boost";

#[cfg(feature = "nvml")]
pub(super) const ARG_NV: &str = "nv";
#[cfg(feature = "nvml")]
pub(super) const ARG_NV_GPU_MIN: &str = "nv-gpu-min";
#[cfg(feature = "nvml")]
pub(super) const ARG_NV_GPU_MAX: &str = "nv-gpu-max";
#[cfg(feature = "nvml")]
pub(super) const ARG_NV_GPU_RESET: &str = "nv-gpu-reset";
#[cfg(feature = "nvml")]
pub(super) const ARG_NV_POWER_LIMIT: &str = "nv-power-limit";

pub(super) const ARG_PROFILE: &str = "PROFILE";

pub(super) const ARG_OPTIONS: &str = "OPTIONS";

const AFTER_HELP: &str = r#"           BOOL   0, 1, true, false
            IDS   comma-delimited sequence of integers and/or integer ranges
          HERTZ*  mhz when unspecified: hz/h - khz/k - mhz/m - ghz/g - thz/t
           SECS   s when unspecified: ns/n - us/u - ms/m - s
        TOGGLES   sequence of 0 (off), 1 (on), or _ (skip), where position denotes id
          WATTS*  w when unspecified: uw/u - mw/m - w - kw/k

        * Floating point values may be given for these units.

    All supported values are shown unless the --show-* or --quiet flags are used.

    All flags may be expressed as env vars. For example:

        --show-cpu        → KNOBS_SHOW_CPU=1
        --cpu 1,3-5       → KNOBS_CPU=1,3-5
        --i915-boost 1200 → KNOBS_I915_BOOST=1200

    The KNOBS_LOG env var may be set to trace, debug, info, warn, or error (default).
"#;

fn app<'a, 'b>() -> clap::App<'a, 'b> {
    let a = App::new(NAME)
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::DisableVersion)
        .setting(AppSettings::TrailingVarArg)
        .setting(AppSettings::UnifiedHelpMessage)
        .version(crate_version!())
        .after_help(AFTER_HELP)
        .arg(
            Arg::with_name(ARG_QUIET)
                .short("q")
                .long(ARG_QUIET)
                .takes_value(false)
                .help("Do not print values"),
        )
        .arg(
            Arg::with_name(ARG_SHOW_CPU)
                .long(ARG_SHOW_CPU)
                .takes_value(false)
                .help("Print cpu and cpufreq values"),
        )
        .arg(
            Arg::with_name(ARG_SHOW_PSTATE)
                .long(ARG_SHOW_PSTATE)
                .takes_value(false)
                .help("Print pstate values"),
        )
        .arg(
            Arg::with_name(ARG_SHOW_RAPL)
                .long(ARG_SHOW_RAPL)
                .takes_value(false)
                .help("Print rapl values"),
        )
        .arg(
            Arg::with_name(ARG_SHOW_I915)
                .long(ARG_SHOW_I915)
                .takes_value(false)
                .help("Print drm values"),
        );

    #[cfg(feature = "nvml")]
    let a = a.arg(
        Arg::with_name(ARG_SHOW_NV)
            .long(ARG_SHOW_NV)
            .takes_value(false)
            .help("Print nvidia values"),
    );

    let a = a
        .arg(
            Arg::with_name(ARG_CPU)
                .short("c")
                .long(ARG_CPU)
                .takes_value(true)
                .value_name("IDS")
                .help("Target cpu ids, default all, ex. 0,1,3-5"),
        )
        .arg(
            Arg::with_name(ARG_CPU_ON)
                .short("o")
                .long(ARG_CPU_ON)
                .takes_value(true)
                .value_name("BOOL")
                .help("Set cpu online status per --cpu"),
        )
        .arg(
            Arg::with_name(ARG_CPU_ON_EACH)
                .short("O")
                .long(ARG_CPU_ON_EACH)
                .takes_value(true)
                .value_name("TOGGLES")
                .help("Set cpu online status, ex. 10_1 → 0:ON 1:OFF 2:SKIP 3:ON"),
        )
        .arg(
            Arg::with_name(ARG_CPUFREQ_GOV)
                .short("g")
                .long(ARG_CPUFREQ_GOV)
                .takes_value(true)
                .value_name("STR")
                .help("Set cpufreq governor per --cpu"),
        )
        .arg(
            Arg::with_name(ARG_CPUFREQ_MIN)
                .short("n")
                .long(ARG_CPUFREQ_MIN)
                .takes_value(true)
                .value_name("HERTZ")
                .help("Set cpufreq min freq per --cpu, ex. 1200 or 1.2ghz"),
        )
        .arg(
            Arg::with_name(ARG_CPUFREQ_MAX)
                .short("x")
                .long(ARG_CPUFREQ_MAX)
                .takes_value(true)
                .value_name("HERTZ")
                .help("Set cpufreq max freq per --cpu, ex. 1200 or 1.2ghz"),
        )
        .arg(
            Arg::with_name(ARG_PSTATE_EPB)
                .long(ARG_PSTATE_EPB)
                .takes_value(true)
                .value_name("0-15")
                .help("Set pstate energy/performance bias per --cpu"),
        )
        .arg(
            Arg::with_name(ARG_PSTATE_EPP)
                .long(ARG_PSTATE_EPP)
                .takes_value(true)
                .value_name("STR")
                .help("Set pstate energy/performance pref per --cpu"),
        )
        .arg(
            Arg::with_name(ARG_RAPL_PACKAGE)
                .short("P")
                .long(ARG_RAPL_PACKAGE)
                .takes_value(true)
                .value_name("INT")
                .help("Target rapl package"),
        )
        .arg(
            Arg::with_name(ARG_RAPL_ZONE)
                .short("Z")
                .long(ARG_RAPL_ZONE)
                .takes_value(true)
                .value_name("INT")
                .help("Target rapl sub-zone"),
        )
        .arg(
            Arg::with_name(ARG_RAPL_LONG_LIMIT)
                .short("L")
                .long(ARG_RAPL_LONG_LIMIT)
                .takes_value(true)
                .value_name("WATTS")
                .help("Set rapl long term power limit per --rapl-package/zone")
                .requires(ARG_RAPL_PACKAGE),
        )
        .arg(
            Arg::with_name(ARG_RAPL_LONG_WINDOW)
                .long(ARG_RAPL_LONG_WINDOW)
                .takes_value(true)
                .value_name("SECS")
                .help("Set rapl long term time window per --rapl-package/zone")
                .requires(ARG_RAPL_PACKAGE),
        )
        .arg(
            Arg::with_name(ARG_RAPL_SHORT_LIMIT)
                .short("S")
                .long(ARG_RAPL_SHORT_LIMIT)
                .takes_value(true)
                .value_name("WATTS")
                .help("Set rapl short term power limit per --rapl-package/zone")
                .requires(ARG_RAPL_PACKAGE),
        )
        .arg(
            Arg::with_name(ARG_RAPL_SHORT_WINDOW)
                .long(ARG_RAPL_SHORT_WINDOW)
                .takes_value(true)
                .value_name("SECS")
                .help("Set rapl short term time window per --rapl-package/zone")
                .requires(ARG_RAPL_PACKAGE),
        )
        .arg(
            Arg::with_name(ARG_I915)
                .long(ARG_I915)
                .takes_value(true)
                .value_name("IDS")
                .help("Target i915 card ids or pci ids, default all, ex. 0,1,3-5"),
        )
        .arg(
            Arg::with_name(ARG_I915_MIN)
                .long(ARG_I915_MIN)
                .takes_value(true)
                .value_name("HERTZ")
                .help("Set i915 min freq per --drm-i915, ex. 1200 or 1.2ghz"),
        )
        .arg(
            Arg::with_name(ARG_I915_MAX)
                .long(ARG_I915_MAX)
                .takes_value(true)
                .value_name("HERTZ")
                .help("Set i915 max freq per --drm-i915, ex. 1200 or 1.2ghz"),
        )
        .arg(
            Arg::with_name(ARG_I915_BOOST)
                .long(ARG_I915_BOOST)
                .takes_value(true)
                .value_name("HERTZ")
                .help("Set i915 boost freq per --drm-i915, ex. 1200 or 1.2ghz"),
        );

    #[cfg(feature = "nvml")]
    let a = a
        .arg(
            Arg::with_name(ARG_NV)
                .long(ARG_NV)
                .takes_value(true)
                .value_name("IDS")
                .help("Target nvidia card ids or pci ids, default all, ex. 0,1,3-5"),
        )
        .arg(
            Arg::with_name(ARG_NV_GPU_MIN)
                .long(ARG_NV_GPU_MIN)
                .takes_value(true)
                .value_name("HERTZ")
                .help("Set nvidia gpu min freq per --nvml, ex. 1200 or 1.2ghz")
                .requires(ARG_NV_GPU_MAX),
        )
        .arg(
            Arg::with_name(ARG_NV_GPU_MAX)
                .long(ARG_NV_GPU_MAX)
                .takes_value(true)
                .value_name("HERTZ")
                .help("Set nvidia gpu max freq per --nvml, ex. 1200 or 1.2ghz")
                .requires(ARG_NV_GPU_MIN),
        )
        .arg(
            Arg::with_name(ARG_NV_GPU_RESET)
                .long(ARG_NV_GPU_RESET)
                .takes_value(false)
                .conflicts_with("nvml-gpu-freq")
                .help("Reset nvidia gpu freq to default per --nvml"),
        )
        .arg(
            Arg::with_name(ARG_NV_POWER_LIMIT)
                .long(ARG_NV_POWER_LIMIT)
                .takes_value(true)
                .value_name("WATTS")
                .help("Set nvidia card power limit per --nvml"),
        );

    let a = a
        .arg(Arg::with_name(ARG_PROFILE))
        .arg(Arg::with_name(ARG_OPTIONS).raw(true));

    a
}

fn var_name(arg: &str) -> String {
    arg.to_uppercase().replace("-", "_")
}

fn var(arg: &str) -> Option<String> {
    let v = env::var(&var_name(arg));
    if let Some(v) = v.as_ref() {
        log::debug!("--{}: using value from environment: {}", arg, v);
    }
    v
}

#[derive(Clone, Debug)]
pub(in crate::cli) struct Parser<'a>(clap::ArgMatches<'a>);

impl<'a> Parser<'a> {
    pub(in crate::cli) fn new(argv: &[String]) -> Result<Self> {
        let m = app().get_matches_from_safe(argv)?;
        Ok(Self(m))
    }

    // Return true if the given argument is present in argv. (Env vars not checked).
    fn present(&self, name: &str) -> bool {
        self.0.is_present(name)
    }

    // Return an iterator over the values for an argument from argv. (Env vars not
    // checked).
    fn values(&self, name: &str) -> Option<clap::Values> {
        self.0.values_of(name)
    }

    // Parse a flag argument from the argv or env vars.
    pub(in crate::cli) fn flag(&self, name: &str) -> Option<()> {
        match self.0.is_present(name) {
            true => Some(()),
            false => match var(name)
                .map(|v| !v.is_empty() && v != "0" && v.to_lowercase() != "false")
                .unwrap_or(false)
            {
                true => Some(()),
                false => None,
            },
        }
    }

    // Parse an integer argument from the argv or env vars.
    pub(in crate::cli) fn int<T>(&self, name: &str) -> Result<Option<T>>
    where
        T: FromStr<Err = std::num::ParseIntError>,
    {
        match self
            .0
            .value_of(name)
            .map(String::from)
            .or_else(|| var(name))
        {
            Some(v) => {
                Ok(Some(T::from_str(&v).map_err(|_| {
                    Error::parse_flag(name, "Expected integer value")
                })?))
            },
            None => Ok(None),
        }
    }

    // Parse a string argument from the argv or env vars.
    pub(in crate::cli) fn str(&self, name: &str) -> Option<String> {
        self.0
            .value_of(name)
            .map(String::from)
            .or_else(|| var(name))
    }

    // Parse an argument using `FromStr` from the argv or env vars.
    pub(in crate::cli) fn from_str<S>(&self, name: &str) -> Result<Option<S>>
    where
        S: FromStr<Err = Error>,
    {
        match self
            .0
            .value_of(name)
            .map(String::from)
            .or_else(|| var(name))
        {
            Some(v) => Ok(Some(
                S::from_str(&v).map_err(|e| Error::parse_flag(name, e))?,
            )),
            None => Ok(None),
        }
    }

    // Parse an argument using `FromStr` from the argv or env vars
    // and convert to the given type.
    pub(in crate::cli) fn from_str_as<S, T>(&self, name: &str) -> Result<Option<T>>
    where
        S: FromStr<Err = Error>,
        T: From<S>,
    {
        Ok(self.from_str::<S>(name)?.map(|v| T::from(v)))
    }
}

impl TryFrom<&Parser<'_>> for Group {
    type Error = Error;

    fn try_from(p: &Parser<'_>) -> Result<Self> {
        let s = Self {
            cpu: p.from_str_as::<Indices, _>(ARG_CPU)?,
            cpu_on: p.from_str_as::<BoolStr, _>(ARG_CPU_ON)?,
            cpu_on_each: p.from_str_as::<Toggles, _>(ARG_CPU_ON_EACH)?,
            cpufreq_gov: p.str(ARG_CPUFREQ_GOV),
            cpufreq_min: p.from_str_as::<FrequencyStr, _>(ARG_CPUFREQ_MIN)?,
            cpufreq_max: p.from_str_as::<FrequencyStr, _>(ARG_CPUFREQ_MAX)?,
            pstate_epb: p.int::<u64>(ARG_PSTATE_EPB)?,
            pstate_epp: p.str(ARG_PSTATE_EPP),
            rapl_package: p.int::<u64>(ARG_RAPL_PACKAGE)?,
            rapl_zone: p.int::<u64>(ARG_RAPL_ZONE)?,
            rapl_long_limit: p.from_str_as::<PowerStr, _>(ARG_RAPL_LONG_LIMIT)?,
            rapl_long_window: p.from_str_as::<DurationStr, _>(ARG_RAPL_LONG_WINDOW)?,
            rapl_short_limit: p.from_str_as::<PowerStr, _>(ARG_RAPL_SHORT_LIMIT)?,
            rapl_short_window: p.from_str_as::<DurationStr, _>(ARG_RAPL_SHORT_WINDOW)?,
            i915: p.from_str_as::<CardIds, _>(ARG_I915)?,
            i915_min: p.from_str_as::<FrequencyStr, _>(ARG_I915_MIN)?,
            i915_max: p.from_str_as::<FrequencyStr, _>(ARG_I915_MAX)?,
            i915_boost: p.from_str_as::<FrequencyStr, _>(ARG_I915_BOOST)?,
            #[cfg(feature = "nvml")]
            nv: p.from_str_as::<CardIds, _>(ARG_NV)?,
            #[cfg(feature = "nvml")]
            nv_gpu_min: p.from_str_as::<FrequencyStr, _>(ARG_NV_GPU_MIN)?,
            #[cfg(feature = "nvml")]
            nv_gpu_max: p.from_str_as::<FrequencyStr, _>(ARG_NV_GPU_MAX)?,
            #[cfg(feature = "nvml")]
            nv_gpu_reset: p.flag(ARG_NV_GPU_RESET).map(|_| true),
            #[cfg(feature = "nvml")]
            nv_power_limit: p.from_str_as::<PowerStr, _>(ARG_NV_POWER_LIMIT)?,
        };
        Ok(s)
    }
}

impl TryFrom<&Parser<'_>> for Groups {
    type Error = Error;

    fn try_from(p: &Parser<'_>) -> Result<Self> {
        let mut groups: Vec<Group> = vec![];
        let mut p = p.clone();
        loop {
            let g = Group::try_from(&p)?;
            if g.has_values() {
                groups.push(g);
            }
            if !p.present(ARG_OPTIONS) {
                break;
            }
            match p.values(ARG_OPTIONS) {
                Some(v) => {
                    let mut v: Vec<String> = v.map(String::from).collect();
                    if v.is_empty() {
                        break;
                    }
                    v.insert(0, NAME.to_string());
                    p = Parser::new(&v)?;
                },
                None => break,
            };
        }
        Ok(groups.into())
    }
}
