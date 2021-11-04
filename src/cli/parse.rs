use crate::{NAME, Chain, Error, Knobs, Result, env};
use crate::cli::*;
use crate::parse::{BoolStr, CardIds, DurationStr, FrequencyStr, Indices, PowerStr, Toggles};
use std::str::FromStr;

// Return the environment variable name for the given cli argument name.
fn var_name(arg: &str) -> String { arg.to_uppercase().replace("-", "_") }

// Return the environment variable value for the given cli argument name.
fn var(arg: &str) -> Option<String> {
    let v = env::var(&var_name(arg));
    if let Some(v) = v.as_ref() {
        log::debug!("--{}: using value from environment: {}", arg, v);
    }
    v
}

// Argument parsing helper.
#[derive(Clone, Debug)]
pub(super) struct Parser<'a>(clap::ArgMatches<'a>);

impl<'a> Parser<'a> {
    pub fn new(argv: &[String]) -> Result<Self> {
        let m = app::build().get_matches_from_safe(argv)?;
        Ok(Self(m))
    }

    // Return true if the given argument is present in argv. (Env vars not checked).
    pub fn arg_present(&self, name: &str) -> bool { self.0.is_present(name) }

    // Return the values for an argument from argv. (Env vars not checked).
    pub fn arg_values(&self, name: &str) -> Option<clap::Values> { self.0.values_of(name) }

    // Parse a flag argument from the argv or from env vars if present.
    pub fn flag(&self, name: &str) -> Option<()> {
        match self.0.is_present(name) {
            true => Some(()),
            false =>
                match var(name)
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
            .or_else(|| var(name))
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
            .or_else(|| var(name))
    }

    // Parse an argument using `FromStr` from the argv or from env vars.
    pub fn from_str<S>(&self, name: &str) -> Result<Option<S>>
    where
        S: FromStr<Err = Error>,
    {
        match self.0.value_of(name)
            .map(String::from)
            .or_else(|| var(name))
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

// Build a `Knobs` from a `Parser`.
impl TryFrom<&Parser<'_>> for Knobs {
    type Error = Error;

    fn try_from(p: &Parser<'_>) -> Result<Self> {
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

// Build a `Chain` from a `Parser`.
impl TryFrom<&Parser<'_>> for Chain {
    type Error = Error;

    fn try_from(p: &Parser<'_>) -> Result<Self> {
        let mut chain: Vec<Knobs> = vec![];
        let mut p = p.clone();
        loop {
            let k = Knobs::try_from(&p)?;
            if k.has_values() { chain.push(k); }
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
}
