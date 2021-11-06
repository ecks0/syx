mod cli;
mod counter;
mod data;
mod de;
mod env;
mod format;
mod logging;
mod parse;
mod path;
mod policy;
mod profile;

pub use clap::{Error as ClapError, ErrorKind as ClapErrorKind};
pub use cli::App;
pub use profile::Error as ProfileError;
pub use tokio::io::{Error as IoError, ErrorKind as IoErrorKind};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Clap(#[from] ClapError),

    #[error(transparent)]
    Format(IoError),

    #[error("--{flag}: {message}")]
    ParseFlag { flag: String, message: String },

    #[error("{0}")]
    ParseValue(String),

    #[error(transparent)]
    Profile(#[from] ProfileError),
}

impl Error {
    fn parse_flag(flag: &str, message: String) -> Self {
        let flag = flag.to_string();
        Self::ParseFlag { flag, message }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

const NAME: &str = "knobs";

use std::collections::HashSet;
use std::time::Duration;

use measurements::{Frequency, Power};
use serde::Deserialize;
use tokio::time::sleep;
use zysfs::types as sysfs;
use zysfs::types::tokio::Write as _;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
enum CardId {
    Id(u64),
    PciId(String),
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, PartialOrd)]
struct Knobs {
    #[serde(default)]
    #[serde(deserialize_with = "de::indices")]
    pub(crate) cpu: Option<Vec<u64>>,

    #[serde(default)]
    #[serde(deserialize_with = "de::bool")]
    pub(crate) cpu_on: Option<bool>,

    #[serde(default)]
    #[serde(deserialize_with = "de::toggles")]
    pub(crate) cpu_on_each: Option<Vec<(u64, bool)>>,

    pub(crate) cpufreq_gov: Option<String>,

    #[serde(default)]
    #[serde(deserialize_with = "de::frequency")]
    pub(crate) cpufreq_min: Option<Frequency>,

    #[serde(default)]
    #[serde(deserialize_with = "de::frequency")]
    pub(crate) cpufreq_max: Option<Frequency>,

    #[serde(default)]
    #[serde(deserialize_with = "de::card_ids")]
    pub(crate) drm_i915: Option<Vec<CardId>>,

    #[serde(default)]
    #[serde(deserialize_with = "de::frequency")]
    pub(crate) drm_i915_min: Option<Frequency>,

    #[serde(default)]
    #[serde(deserialize_with = "de::frequency")]
    pub(crate) drm_i915_max: Option<Frequency>,

    #[serde(default)]
    #[serde(deserialize_with = "de::frequency")]
    pub(crate) drm_i915_boost: Option<Frequency>,

    #[cfg(feature = "nvml")]
    #[serde(default)]
    #[serde(deserialize_with = "de::card_ids")]
    pub(crate) nvml: Option<Vec<CardId>>,

    #[cfg(feature = "nvml")]
    #[serde(default)]
    #[serde(deserialize_with = "de::frequency")]
    pub(crate) nvml_gpu_min: Option<Frequency>,

    #[cfg(feature = "nvml")]
    #[serde(default)]
    #[serde(deserialize_with = "de::frequency")]
    pub(crate) nvml_gpu_max: Option<Frequency>,

    #[cfg(feature = "nvml")]
    #[serde(default)]
    #[serde(deserialize_with = "de::bool")]
    pub(crate) nvml_gpu_reset: Option<bool>,

    #[cfg(feature = "nvml")]
    #[serde(default)]
    #[serde(deserialize_with = "de::power")]
    pub(crate) nvml_power_limit: Option<Power>,

    pub(crate) pstate_epb: Option<u64>,

    pub(crate) pstate_epp: Option<String>,

    pub(crate) rapl_package: Option<u64>,

    pub(crate) rapl_zone: Option<u64>,

    #[serde(default)]
    #[serde(deserialize_with = "de::power")]
    pub(crate) rapl_long_limit: Option<Power>,

    #[serde(default)]
    #[serde(deserialize_with = "de::duration")]
    pub(crate) rapl_long_window: Option<Duration>,

    #[serde(default)]
    #[serde(deserialize_with = "de::power")]
    pub(crate) rapl_short_limit: Option<Power>,

    #[serde(default)]
    #[serde(deserialize_with = "de::duration")]
    pub(crate) rapl_short_window: Option<Duration>,
}

impl Knobs {
    pub(crate) fn has_cpu_values(&self) -> bool {
        self.cpu_on.is_some() || self.cpu_on_each.is_some()
    }

    pub(crate) fn has_cpu_related_values(&self) -> bool {
        self.has_cpu_values() || self.has_cpufreq_values() || self.has_pstate_values()
    }

    pub(crate) fn has_cpufreq_values(&self) -> bool {
        self.cpufreq_gov.is_some() || self.cpufreq_min.is_some() || self.cpufreq_max.is_some()
    }

    pub(crate) fn has_drm_i915_values(&self) -> bool {
        self.drm_i915_min.is_some() || self.drm_i915_max.is_some() || self.drm_i915_boost.is_some()
    }

    pub(crate) fn has_drm_values(&self) -> bool { self.has_drm_i915_values() }

    #[cfg(feature = "nvml")]
    pub(crate) fn has_nvml_values(&self) -> bool {
        self.nvml_gpu_min.is_some()
            || self.nvml_gpu_max.is_some()
            || self.nvml_gpu_reset.is_some()
            || self.nvml_power_limit.is_some()
    }

    pub(crate) fn has_pstate_values(&self) -> bool {
        self.pstate_epb.is_some() || self.pstate_epp.is_some()
    }

    pub(crate) fn has_rapl_values(&self) -> bool {
        self.rapl_long_limit.is_some()
            || self.rapl_long_window.is_some()
            || self.rapl_short_limit.is_some()
            || self.rapl_short_window.is_some()
    }

    pub(crate) fn has_values(&self) -> bool {
        let b = self.has_cpu_related_values() || self.has_drm_values() || self.has_rapl_values();
        #[cfg(feature = "nvml")]
        let b = b || self.has_nvml_values();
        b
    }

    async fn resolve(&mut self) {
        if self.has_cpu_related_values() && self.cpu.is_none() {
            self.cpu = policy::cpu_ids().await;
        }
        if self.has_drm_i915_values() && self.drm_i915.is_none() {
            self.drm_i915 =
                policy::drm_i915_ids().await.map(|ids| ids.into_iter().map(CardId::Id).collect());
        }
        #[cfg(feature = "nvml")]
        if self.has_nvml_values() && self.nvml.is_none() {
            self.nvml =
                policy::nvml_ids().await.map(|ids| ids.into_iter().map(CardId::Id).collect());
        }
    }

    async fn apply_cpu(&self) {
        let cpu: Option<sysfs::cpu::Cpu> = self.into();
        if let Some(cpu) = cpu {
            cpu.write().await;
        };
    }

    async fn apply_cpufreq(&self) {
        let cpufreq: Option<sysfs::cpufreq::Cpufreq> = self.into();
        if let Some(cpufreq) = cpufreq {
            cpufreq.write().await;
        };
    }

    async fn apply_drm(&self) {
        let drm: Option<sysfs::drm::Drm> = self.into();
        if let Some(drm) = drm {
            drm.write().await;
        };
    }

    #[cfg(feature = "nvml")]
    async fn apply_nvml(&self) {
        let nvml: Option<policy::NvmlPolicies> = self.into();
        if let Some(nvml) = nvml {
            nvml.write();
        }
    }

    async fn apply_pstate(&self) {
        let intel_pstate: Option<sysfs::intel_pstate::IntelPstate> = self.into();
        if let Some(intel_pstate) = intel_pstate {
            intel_pstate.write().await;
        }
    }

    async fn apply_rapl(&self) {
        let intel_rapl: Option<sysfs::intel_rapl::IntelRapl> = self.into();
        if let Some(intel_rapl) = intel_rapl {
            intel_rapl.write().await;
        }
    }
}

#[derive(Clone, Debug)]
struct Chain {
    knobses: Vec<Knobs>,
}

impl Chain {
    const CPU_ONOFFLINE_WAIT: Duration = Duration::from_millis(300);
    const CPU_RELATED_WAIT: Duration = Duration::from_millis(100);

    fn has_cpufreq_values(&self) -> bool { self.knobses.iter().any(|k| k.has_cpufreq_values()) }

    fn has_pstate_values(&self) -> bool { self.knobses.iter().any(|k| k.has_pstate_values()) }

    async fn cpu_onoff_wait() { sleep(Self::CPU_ONOFFLINE_WAIT).await }

    async fn cpu_related_wait() { sleep(Self::CPU_RELATED_WAIT).await }

    fn cpu_ids_for_chain(&self) -> Vec<u64> {
        let mut ids: Vec<u64> = self
            .knobses
            .iter()
            .fold(HashSet::new(), |mut h, k| {
                if let Some(ids) = k.cpu.clone() {
                    h.extend(ids.into_iter());
                };
                h
            })
            .into_iter()
            .collect();
        ids.sort_unstable();
        ids
    }

    async fn cpus_online_all(&self) -> Vec<u64> {
        let cpu_ids = self.cpu_ids_for_chain();
        let cpu_ids = policy::set_cpus_online(cpu_ids).await;
        if !cpu_ids.is_empty() {
            Self::cpu_onoff_wait().await;
        }
        cpu_ids
    }

    async fn cpus_online_reset(&self, cpu_ids: Vec<u64>) {
        let cpu_ids = policy::set_cpus_offline(cpu_ids).await;
        if !cpu_ids.is_empty() {
            Self::cpu_onoff_wait().await;
        }
    }

    pub(crate) fn has_values(&self) -> bool { self.knobses.iter().any(|k| k.has_values()) }

    pub(crate) async fn resolve(&mut self) {
        for k in self.knobses.iter_mut() {
            k.resolve().await;
        }
    }

    pub(crate) async fn apply(&self) {
        if log::log_enabled!(log::Level::Trace) {
            for (i, k) in self.knobses.iter().enumerate() {
                log::trace!("Group {}", i);
                log::trace!("{:#?}", k);
            }
        }
        if self.has_cpufreq_values() || self.has_pstate_values() {
            let onlined = self.cpus_online_all().await;

            for (i, k) in self.knobses.iter().enumerate() {
                log::debug!("Group {} Pass 0", i);
                k.apply_cpufreq().await;
                k.apply_pstate().await;
                Self::cpu_related_wait().await;
            }

            self.cpus_online_reset(onlined).await;
        }
        for (i, k) in self.knobses.iter().enumerate() {
            log::debug!("Group {} Pass 1", i);
            k.apply_cpu().await;
            if k.has_cpu_values() {
                Self::cpu_onoff_wait().await;
            }
            k.apply_rapl().await;
            k.apply_drm().await;
            #[cfg(feature = "nvml")]
            k.apply_nvml().await;
        }
    }
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
    type IntoIter = std::vec::IntoIter<Knobs>;
    type Item = Knobs;

    fn into_iter(self) -> Self::IntoIter { self.knobses.into_iter() }
}
