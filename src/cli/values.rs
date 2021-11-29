use std::str::FromStr;
use std::time::Duration;

use measurements::{Frequency, Power};
use serde::de::Error as _;
use serde::{Deserialize, Deserializer};
use tokio::sync::OnceCell;

use crate::cli::{Error, Result};
#[cfg(feature = "nvml")]
use crate::nvml;
use crate::{sysfs, Values as _};

async fn cpu_ids() -> &'static Vec<u64> {
    static CPU_IDS: OnceCell<Vec<u64>> = OnceCell::const_new();
    async fn cpu_ids() -> Vec<u64> {
        sysfs::cpu::Device::ids().await
    }
    CPU_IDS.get_or_init(cpu_ids).await
}

async fn i915_ids() -> &'static Vec<u64> {
    static I915_IDS: OnceCell<Vec<u64>> = OnceCell::const_new();
    async fn i915_ids() -> Vec<u64> {
        sysfs::i915::Device::ids().await
    }
    I915_IDS.get_or_init(i915_ids).await
}

#[cfg(feature = "nvml")]
async fn nvml_ids() -> &'static Vec<u32> {
    static NVML_IDS: OnceCell<Vec<u32>> = OnceCell::const_new();
    async fn nvml_ids() -> Vec<u32> {
        crate::nvml::Device::ids().await
    }
    NVML_IDS.get_or_init(nvml_ids).await
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
pub(in crate::cli) enum CardId {
    Index(u64),
    PciId(String),
}

impl FromStr for CardId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.contains(':') {
            Ok(Self::PciId(s.into()))
        } else {
            let id = s
                .parse::<u64>()
                .map_err(|_| Error::parse_value("Expected id integer or pci id string"))?;
            Ok(Self::Index(id))
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
pub(in crate::cli) struct Values {
    #[serde(default)]
    #[serde(deserialize_with = "de::indices")]
    pub(in crate::cli) cpu: Option<Vec<u64>>,

    #[serde(default)]
    #[serde(deserialize_with = "de::bool")]
    pub(in crate::cli) cpu_on: Option<bool>,

    #[serde(default)]
    #[serde(deserialize_with = "de::toggles")]
    pub(in crate::cli) cpu_on_each: Option<Vec<(u64, bool)>>,

    pub(in crate::cli) cpufreq_gov: Option<String>,

    #[serde(default)]
    #[serde(deserialize_with = "de::frequency")]
    pub(in crate::cli) cpufreq_min: Option<Frequency>,

    #[serde(default)]
    #[serde(deserialize_with = "de::frequency")]
    pub(in crate::cli) cpufreq_max: Option<Frequency>,

    pub(in crate::cli) pstate_epb: Option<u64>,

    pub(in crate::cli) pstate_epp: Option<String>,

    pub(in crate::cli) rapl_package: Option<u64>,

    pub(in crate::cli) rapl_zone: Option<u64>,

    pub(in crate::cli) rapl_constraint: Option<u64>,

    #[serde(default)]
    #[serde(deserialize_with = "de::power")]
    pub(in crate::cli) rapl_limit: Option<Power>,

    #[serde(default)]
    #[serde(deserialize_with = "de::duration")]
    pub(in crate::cli) rapl_window: Option<Duration>,

    #[serde(default)]
    #[serde(deserialize_with = "de::card_ids")]
    pub(in crate::cli) i915: Option<Vec<CardId>>,

    #[serde(default)]
    #[serde(deserialize_with = "de::frequency")]
    pub(in crate::cli) i915_min: Option<Frequency>,

    #[serde(default)]
    #[serde(deserialize_with = "de::frequency")]
    pub(in crate::cli) i915_max: Option<Frequency>,

    #[serde(default)]
    #[serde(deserialize_with = "de::frequency")]
    pub(in crate::cli) i915_boost: Option<Frequency>,

    #[cfg(feature = "nvml")]
    #[serde(default)]
    #[serde(deserialize_with = "de::card_ids")]
    pub(in crate::cli) nv: Option<Vec<CardId>>,

    #[cfg(feature = "nvml")]
    #[serde(default)]
    #[serde(deserialize_with = "de::frequency")]
    pub(in crate::cli) nv_gpu_min: Option<Frequency>,

    #[cfg(feature = "nvml")]
    #[serde(default)]
    #[serde(deserialize_with = "de::frequency")]
    pub(in crate::cli) nv_gpu_max: Option<Frequency>,

    #[cfg(feature = "nvml")]
    #[serde(default)]
    #[serde(deserialize_with = "de::bool")]
    pub(in crate::cli) nv_gpu_reset: Option<bool>,

    #[cfg(feature = "nvml")]
    #[serde(default)]
    #[serde(deserialize_with = "de::power")]
    pub(in crate::cli) nv_power_limit: Option<Power>,
}

impl Values {
    pub(in crate::cli) fn has_cpu_related_values(&self) -> bool {
        self.has_cpu_values() || self.has_cpufreq_values() || self.has_pstate_values()
    }

    pub(in crate::cli) fn has_cpu_values(&self) -> bool {
        self.cpu_on.is_some() || self.cpu_on_each.is_some()
    }

    pub(in crate::cli) fn has_cpufreq_values(&self) -> bool {
        self.cpufreq_gov.is_some() || self.cpufreq_min.is_some() || self.cpufreq_max.is_some()
    }

    pub(in crate::cli) fn has_pstate_values(&self) -> bool {
        self.pstate_epb.is_some() || self.pstate_epp.is_some()
    }

    pub(in crate::cli) fn has_rapl_values(&self) -> bool {
        self.rapl_limit.is_some() || self.rapl_window.is_some()
    }

    pub(in crate::cli) fn has_i915_values(&self) -> bool {
        self.i915_min.is_some() || self.i915_max.is_some() || self.i915_boost.is_some()
    }

    #[cfg(feature = "nvml")]
    pub(in crate::cli) fn has_nvml_values(&self) -> bool {
        self.nv_gpu_min.is_some()
            || self.nv_gpu_max.is_some()
            || self.nv_gpu_reset.is_some()
            || self.nv_power_limit.is_some()
    }

    #[allow(clippy::let_and_return)]
    pub(in crate::cli) fn has_values(&self) -> bool {
        let b = self.has_cpu_related_values() || self.has_i915_values() || self.has_rapl_values();
        #[cfg(feature = "nvml")]
        let b = b || self.has_nvml_values();
        b
    }

    async fn as_cpu(&self) -> Option<sysfs::Cpu> {
        if !self.has_cpu_values() {
            return None;
        }
        let cpu = match self.cpu.as_ref() {
            Some(v) => v,
            None => cpu_ids().await,
        }.clone();
        let mut devices: Vec<sysfs::cpu::Device> = cpu
            .into_iter()
            .map(|id| {
                let online = self.cpu_on;
                sysfs::cpu::Device { id, online }
            })
            .collect();
        if let Some(cpu_on_each) = self.cpu_on_each.clone() {
            for (id, online) in cpu_on_each {
                if let Some(mut p) = devices.iter_mut().find(|p| p.id == id) {
                    p.online = Some(online);
                } else {
                    let online = Some(online);
                    let d = sysfs::cpu::Device { id, online };
                    devices.push(d);
                }
            }
        }
        devices.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        let r = sysfs::Cpu { devices };
        Some(r)
    }

    async fn as_cpufreq(&self) -> Option<sysfs::Cpufreq> {
        if !self.has_cpufreq_values() {
            return None;
        }
        let cpu = match self.cpu.as_ref() {
            Some(v) => v,
            None => cpu_ids().await,
        }.clone();
        let scaling_min_freq = self.cpufreq_min.map(|f| f.as_kilohertz().round() as u64);
        let scaling_max_freq = self.cpufreq_max.map(|f| f.as_kilohertz().round() as u64);
        let devices: Vec<sysfs::cpufreq::Device> = cpu
            .into_iter()
            .map(|id| {
                let scaling_governor = self.cpufreq_gov.clone();
                sysfs::cpufreq::Device {
                    id,
                    scaling_governor,
                    scaling_min_freq,
                    scaling_max_freq,
                    ..Default::default()
                }
            })
            .collect();
        let r = sysfs::Cpufreq { devices };
        Some(r)
    }
    async fn as_intel_pstate(&self) -> Option<sysfs::IntelPstate> {
        if !self.has_pstate_values() {
            return None;
        }
        let cpu = match self.cpu.as_ref() {
            Some(v) => v,
            None => cpu_ids().await,
        }.clone();
        let energy_perf_bias = self.pstate_epb;
        let devices: Vec<sysfs::intel_pstate::Device> = cpu
            .into_iter()
            .map(|id| {
                let energy_performance_preference = self.pstate_epp.clone();
                sysfs::intel_pstate::Device {
                    id,
                    energy_perf_bias,
                    energy_performance_preference,
                    ..Default::default()
                }
            })
            .collect();
        let r = sysfs::intel_pstate::IntelPstate {
            devices,
            ..Default::default()
        };
        Some(r)
    }

    async fn as_intel_rapl(&self) -> Option<sysfs::IntelRapl> {
        let zone = self.rapl_package?;
        let id = self.rapl_constraint?;
        let subzone = self.rapl_zone;
        let power_limit_uw = self.rapl_limit.map(|v| v.as_microwatts().round() as u64);
        let time_window_us = self.rapl_window.map(|v| v.as_micros().try_into().unwrap());
        let constraint = sysfs::intel_rapl::Constraint {
            id,
            power_limit_uw,
            time_window_us,
            ..Default::default()
        };
        let id = sysfs::intel_rapl::ZoneId { zone, subzone };
        let constraints = vec![constraint];
        let device = sysfs::intel_rapl::Device {
            id,
            constraints,
            ..Default::default()
        };
        let devices = vec![device];
        let r = sysfs::IntelRapl { devices };
        Some(r)
    }

    async fn as_i915(&self) -> Option<sysfs::I915> {
        if !self.has_i915_values() {
            return None;
        }
        let i915 = match self.i915.as_ref() {
            Some(v) => v.clone(),
            None => i915_ids()
                .await
                .clone()
                .into_iter()
                .map(|id| CardId::Index(id as u64))
                .collect()
        };
        let min_freq_mhz = self.i915_min.map(|f| f.as_megahertz().round() as u64);
        let max_freq_mhz = self.i915_max.map(|f| f.as_megahertz().round() as u64);
        let boost_freq_mhz = self.i915_boost.map(|f| f.as_megahertz().round() as u64);
        let devices: Vec<sysfs::i915::Device> = i915
            .into_iter()
            .map(|id| {
                let id = match id {
                    CardId::Index(id) => id,
                    CardId::PciId(_) => {
                        panic!("Indexing i915 devices by PCI ID is not yet implemented")
                    },
                };
                sysfs::i915::Device {
                    id,
                    min_freq_mhz,
                    max_freq_mhz,
                    boost_freq_mhz,
                    ..Default::default()
                }
            })
            .collect();
        let r = sysfs::I915 { devices };
        Some(r)
    }

    #[cfg(feature = "nvml")]
    async fn as_nvml(&self) -> Option<nvml::Nvml> {
        if !self.has_nvml_values() {
            return None;
        }
        let nv = match self.nv.as_ref() {
            Some(v) => v.clone(),
            None => nvml_ids()
                .await
                .clone()
                .into_iter()
                .map(|id| CardId::Index(id as u64))
                .collect(),
        };
        let gfx_freq_min = self.nv_gpu_min.map(|f| f.as_megahertz().round() as u32);
        let gfx_freq_max = self.nv_gpu_max.map(|f| f.as_megahertz().round() as u32);
        let gfx_freq_reset = self.nv_gpu_reset;
        let power_limit = self
            .nv_power_limit
            .map(|p| p.as_milliwatts().round() as u32);
        let devices: Vec<nvml::Device> = nv
            .into_iter()
            .map(|id| {
                let id = match id {
                    CardId::Index(id) => id.try_into().unwrap(),
                    CardId::PciId(_) => {
                        panic!("Indexing nvml devices by PCI ID is not yet implemented")
                    },
                };
                nvml::Device {
                    id,
                    gfx_freq_min,
                    gfx_freq_max,
                    gfx_freq_reset,
                    power_limit,
                    ..Default::default()
                }
            })
            .collect();
        let r = nvml::Nvml { devices };
        Some(r)
    }

    pub(in crate::cli) async fn as_machine(&self) -> crate::Machine {
        let cpu = self.as_cpu().await;
        let cpufreq = self.as_cpufreq().await;
        let intel_pstate = self.as_intel_pstate().await;
        let intel_rapl = self.as_intel_rapl().await;
        let i915 = self.as_i915().await;
        #[cfg(feature = "nvml")]
        let nvml = self.as_nvml().await;
        crate::Machine {
            cpu,
            cpufreq,
            i915,
            intel_pstate,
            intel_rapl,
            #[cfg(feature = "nvml")]
            nvml,
        }
    }
}

mod de {
    use std::time::Duration;

    use measurements::{Frequency, Power};
    use serde::{Deserialize, Deserializer};

    use crate::cli::parse::{
        BoolStr,
        CardIds,
        DurationStr,
        FrequencyStr,
        Indices,
        PowerStr,
        Toggles,
    };
    use crate::cli::values::CardId;

    pub(super) fn bool<'de, D>(deserializer: D) -> std::result::Result<Option<bool>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: BoolStr = Deserialize::deserialize(deserializer)?;
        Ok(Some(v.into()))
    }

    pub(super) fn card_ids<'de, D>(
        deserializer: D,
    ) -> std::result::Result<Option<Vec<CardId>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: CardIds = Deserialize::deserialize(deserializer)?;
        Ok(Some(v.into()))
    }

    pub(super) fn duration<'de, D>(
        deserializer: D,
    ) -> std::result::Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: DurationStr = Deserialize::deserialize(deserializer)?;
        Ok(Some(v.into()))
    }

    pub(super) fn frequency<'de, D>(
        deserializer: D,
    ) -> std::result::Result<Option<Frequency>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: FrequencyStr = Deserialize::deserialize(deserializer)?;
        Ok(Some(v.into()))
    }

    pub(super) fn indices<'de, D>(
        deserializer: D,
    ) -> std::result::Result<Option<Vec<u64>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: Indices = Deserialize::deserialize(deserializer)?;
        Ok(Some(v.into()))
    }

    pub(super) fn power<'de, D>(deserializer: D) -> std::result::Result<Option<Power>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: PowerStr = Deserialize::deserialize(deserializer)?;
        Ok(Some(v.into()))
    }

    pub(super) fn toggles<'de, D>(
        deserializer: D,
    ) -> std::result::Result<Option<Vec<(u64, bool)>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: Toggles = Deserialize::deserialize(deserializer)?;
        Ok(Some(v.into()))
    }
}
