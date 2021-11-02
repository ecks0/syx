use measurements::{Frequency, Power};
use serde::{Deserialize, Deserializer, de::Error as _};
use zysfs::types::{self as sysfs, tokio::Write as _};
use tokio::time::sleep;
use std::{collections::HashSet, str::FromStr, time::Duration};

pub mod cli;
pub mod data;
pub mod format;
pub mod parse;
pub mod policy;

pub use format::FormatValues;

pub use measurements;
#[cfg(feature = "nvml")]
pub use nvml_facade;
pub use zysfs;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Clap(#[from] clap::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("--{flag}: {message}")]
    ParseFlag {
        flag: String,
        message: String,
    },

    #[error("{0}")]
    ParseValue(String),

    #[error(transparent)]
    Profile(#[from] cli::ProfileError),
}

impl Error {
    fn parse_flag(flag: &str, message: String) -> Self {
        Self::ParseFlag {
            flag: flag.to_string(),
            message,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

// `Knobs` deserialization helpers
mod de {
    use super::{CardId, Deserialize, Deserializer, Duration, Frequency, Power, parse};

    pub fn bool<'de, D>(deserializer: D) -> std::result::Result<Option<bool>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: parse::BoolStr = Deserialize::deserialize(deserializer)?;
        Ok(Some(v.into()))
    }

    pub fn card_ids<'de, D>(deserializer: D) -> std::result::Result<Option<Vec<CardId>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: parse::CardIds = Deserialize::deserialize(deserializer)?;
        Ok(Some(v.into()))
    }

    pub fn frequency<'de, D>(deserializer: D) -> std::result::Result<Option<Frequency>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: parse::FrequencyStr = Deserialize::deserialize(deserializer)?;
        Ok(Some(v.into()))
    }

    pub fn indices<'de, D>(deserializer: D) -> std::result::Result<Option<Vec<u64>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: parse::Indices = Deserialize::deserialize(deserializer)?;
        Ok(Some(v.into()))
    }

    pub fn toggles<'de, D>(deserializer: D) -> std::result::Result<Option<Vec<(u64, bool)>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: parse::Toggles = Deserialize::deserialize(deserializer)?;
        Ok(Some(v.into()))
    }

    pub fn power<'de, D>(deserializer: D) -> std::result::Result<Option<Power>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: parse::PowerStr = Deserialize::deserialize(deserializer)?;
        Ok(Some(v.into()))
    }

    pub fn duration<'de, D>(deserializer: D) -> std::result::Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: parse::DurationStr = Deserialize::deserialize(deserializer)?;
        Ok(Some(v.into()))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
pub enum CardId {
    Id(u64),
    PciId(String),
}

impl FromStr for CardId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.contains(':') {
            Ok(Self::PciId(s.into()))
        } else {
            let id = s.parse::<u64>()
                .map_err(|_| Error::ParseValue("Expected id integer or pci id string".into()))?;
            Ok(Self::Id(id))
        }
    }
}

impl<'de> Deserialize<'de> for CardId {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Self::from_str(&s).map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, PartialOrd)]
pub struct Knobs {

    #[serde(default)]
    #[serde(deserialize_with = "de::indices")]
    pub cpu: Option<Vec<u64>>,

    #[serde(default)]
    #[serde(deserialize_with = "de::bool")]
    pub cpu_on: Option<bool>,

    #[serde(default)]
    #[serde(deserialize_with = "de::toggles")]
    pub cpu_on_each: Option<Vec<(u64, bool)>>,

    pub cpufreq_gov: Option<String>,

    #[serde(default)]
    #[serde(deserialize_with = "de::frequency")]
    pub cpufreq_min: Option<Frequency>,

    #[serde(default)]
    #[serde(deserialize_with = "de::frequency")]
    pub cpufreq_max: Option<Frequency>,

    #[serde(default)]
    #[serde(deserialize_with = "de::card_ids")]
    pub drm_i915: Option<Vec<CardId>>,

    #[serde(default)]
    #[serde(deserialize_with = "de::frequency")]
    pub drm_i915_min: Option<Frequency>,

    #[serde(default)]
    #[serde(deserialize_with = "de::frequency")]
    pub drm_i915_max: Option<Frequency>,

    #[serde(default)]
    #[serde(deserialize_with = "de::frequency")]
    pub drm_i915_boost: Option<Frequency>,

    #[cfg(feature = "nvml")]
    #[serde(default)]
    #[serde(deserialize_with = "de::card_ids")]
    pub nvml: Option<Vec<CardId>>,

    #[cfg(feature = "nvml")]
    #[serde(default)]
    #[serde(deserialize_with = "de::frequency")]
    pub nvml_gpu_min: Option<Frequency>,

    #[cfg(feature = "nvml")]
    #[serde(default)]
    #[serde(deserialize_with = "de::frequency")]
    pub nvml_gpu_max: Option<Frequency>,

    #[cfg(feature = "nvml")]
    #[serde(default)]
    #[serde(deserialize_with = "de::bool")]
    pub nvml_gpu_reset: Option<bool>,

    #[cfg(feature = "nvml")]
    #[serde(default)]
    #[serde(deserialize_with = "de::power")]
    pub nvml_power_limit: Option<Power>,

    pub pstate_epb: Option<u64>,

    pub pstate_epp: Option<String>,

    pub rapl_package: Option<u64>,

    pub rapl_zone: Option<u64>,

    #[serde(default)]
    #[serde(deserialize_with = "de::power")]
    pub rapl_long_limit: Option<Power>,

    #[serde(default)]
    #[serde(deserialize_with = "de::duration")]
    pub rapl_long_window: Option<Duration>,

    #[serde(default)]
    #[serde(deserialize_with = "de::power")]
    pub rapl_short_limit: Option<Power>,

    #[serde(default)]
    #[serde(deserialize_with = "de::duration")]
    pub rapl_short_window: Option<Duration>,
}

impl Knobs {
    pub fn has_cpu_values(&self) -> bool {
        self.cpu_on.is_some() ||
        self.cpu_on_each.is_some()
    }

    pub fn has_cpu_related_values(&self) -> bool {
        self.has_cpu_values() ||
        self.has_cpufreq_values() ||
        self.has_pstate_values()
    }

    pub fn has_cpufreq_values(&self) -> bool {
        self.cpufreq_gov.is_some() ||
        self.cpufreq_min.is_some() ||
        self.cpufreq_max.is_some()
    }

    pub fn has_drm_i915_values(&self) -> bool {
        self.drm_i915_min.is_some() ||
        self.drm_i915_max.is_some() ||
        self.drm_i915_boost.is_some()
    }

    pub fn has_drm_values(&self) -> bool {
        self.has_drm_i915_values()
    }

    #[cfg(feature = "nvml")]
    pub fn has_nvml_values(&self) -> bool {
        self.nvml_gpu_min.is_some() ||
        self.nvml_gpu_max.is_some() ||
        self.nvml_gpu_reset.is_some() ||
        self.nvml_power_limit.is_some()
    }

    pub fn has_pstate_values(&self) -> bool {
        self.pstate_epb.is_some() ||
        self.pstate_epp.is_some()
    }

    pub fn has_rapl_values(&self) -> bool {
        self.rapl_long_limit.is_some() ||
        self.rapl_long_window.is_some() ||
        self.rapl_short_limit.is_some() ||
        self.rapl_short_window.is_some()
    }

    pub async fn apply_cpu_values(&self) {
        let cpu: Option<sysfs::cpu::Cpu> = self.into();
        if let Some(cpu) = cpu { cpu.write().await; };
    }

    pub async fn apply_cpufreq_values(&self) {
        let cpufreq: Option<sysfs::cpufreq::Cpufreq> = self.into();
        if let Some(cpufreq) = cpufreq { cpufreq.write().await; };
    }

    pub async fn apply_drm_values(&self) {
        let drm: Option<sysfs::drm::Drm> = self.into();
        if let Some(drm) = drm { drm.write().await; };
    }

    #[cfg(feature = "nvml")]
    pub async fn apply_nvml_values(&self) {
        let nvml: Option<policy::NvmlPolicies> = self.into();
        if let Some(nvml) = nvml { nvml.write(); }
    }

    pub async fn apply_pstate_values(&self) {
        let intel_pstate: Option<sysfs::intel_pstate::IntelPstate> = self.into();
        if let Some(intel_pstate) = intel_pstate { intel_pstate.write().await; }
    }

    pub async fn apply_rapl_values(&self) {
        let intel_rapl: Option<sysfs::intel_rapl::IntelRapl> = self.into();
        if let Some(intel_rapl) = intel_rapl { intel_rapl.write().await; }
    }

    pub async fn apply_values(&self) { Chain::from(self.clone()).apply_values().await; }
}

#[derive(Clone, Debug)]
pub struct Chain {
    knobses: Vec<Knobs>,
}

impl Chain {

    pub fn has_cpu_values(&self) -> bool { self.knobses.iter().any(|k| k.has_cpu_values()) }

    pub fn has_cpu_related_values(&self) -> bool { self.knobses.iter().any(|k| k.has_cpu_related_values()) }

    pub fn has_cpufreq_values(&self) -> bool { self.knobses.iter().any(|k| k.has_cpufreq_values()) }

    pub fn has_drm_i915_values(&self) -> bool { self.knobses.iter().any(|k| k.has_drm_i915_values()) }

    pub fn has_drm_values(&self) -> bool { self.knobses.iter().any(|k| k.has_drm_values()) }

    #[cfg(feature = "nvml")]
    pub fn has_nvml_values(&self) -> bool { self.knobses.iter().any(|k| k.has_nvml_values()) }

    pub fn has_pstate_values(&self) -> bool { self.knobses.iter().any(|k| k.has_pstate_values()) }

    pub fn has_rapl_values(&self) -> bool { self.knobses.iter().any(|k| k.has_rapl_values()) }

    const CPU_ONOFFLINE_WAIT: Duration = Duration::from_millis(250);

    async fn cpu_onoff_wait() { sleep(Self::CPU_ONOFFLINE_WAIT).await }

    fn cpu_ids_for_chain(&self) -> Vec<u64> {
        let mut ids: Vec<u64> = self.knobses
            .iter()
            .fold(
                HashSet::new(),
                |mut h, k| { if let Some(ids) = k.cpu.clone() { h.extend(ids.into_iter()); }; h })
            .into_iter()
            .collect();
        ids.sort_unstable();
        ids
    }

    async fn cpus_online_all(&self) -> Vec<u64> {
        let cpu_ids = self.cpu_ids_for_chain();
        let cpu_ids = policy::set_cpus_online(cpu_ids).await;
        if !cpu_ids.is_empty() { Self::cpu_onoff_wait().await; }
        cpu_ids
    }

    async fn cpus_online_reset(&self, cpu_ids: Vec<u64>) {
        let cpu_ids = policy::set_cpus_offline(cpu_ids).await;
        if !cpu_ids.is_empty() { Self::cpu_onoff_wait().await; }
    }

    pub async fn apply_values(&self) {
        if log::log_enabled!(log::Level::Trace) {
            for (i, k) in self.iter().enumerate() {
                log::trace!("Group {}", i);
                log::trace!("{:#?}", k);
            }
        }
        if self.has_cpufreq_values() || self.has_pstate_values() {
            let onlined = self.cpus_online_all().await;

            for (i, k) in self.knobses.iter().enumerate() {
                log::info!("Group {} Pass 0", i);
                k.apply_cpufreq_values().await;
                k.apply_pstate_values().await;
            }

            self.cpus_online_reset(onlined).await;
        }
        for (i, k) in self.knobses.iter().enumerate() {
            log::info!("Group {} Pass 1", i);
            k.apply_cpu_values().await;
            if k.has_cpu_values() { Self::cpu_onoff_wait().await; }
            k.apply_rapl_values().await;
            k.apply_drm_values().await;
            #[cfg(feature = "nvml")]
            k.apply_nvml_values().await;
        }
    }

    pub fn iter(&self) -> impl Iterator<Item=&Knobs> { self.knobses.iter() }

    pub fn iter_mut(&mut self) -> impl Iterator<Item=&mut Knobs> { self.knobses.iter_mut() }
}

impl From<Vec<Knobs>> for Chain {
    fn from(knobses: Vec<Knobs>) -> Self { Self { knobses } }
}

impl From<Knobs> for Chain {
    fn from(knobs: Knobs) -> Self { Self::from(vec![knobs]) }
}

impl From<Chain> for Vec<Knobs> {
    fn from(c: Chain) -> Self { c.knobses }
}

impl IntoIterator for Chain {
    type Item = crate::Knobs;
    type IntoIter = std::vec::IntoIter<Knobs>;

    fn into_iter(self) -> Self::IntoIter { self.knobses.into_iter() }
}

impl<'de> Deserialize<'de> for Chain {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Vec<Knobs> = Deserialize::deserialize(deserializer)?;
        Ok(s.into())
    }
}
