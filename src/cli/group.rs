use std::collections::HashSet;
use std::time::Duration;

use measurements::{Frequency, Power};
use serde::Deserialize;
use tokio::sync::OnceCell;
use tokio::time::sleep;

use crate::cli::de;
use crate::cli::policy::ToPolicy;
#[cfg(feature = "nvml")]
use crate::nvml;
use crate::{sysfs, Policy as _};

async fn cpu_ids() -> Vec<u64> {
    static CPU_IDS: OnceCell<Vec<u64>> = OnceCell::const_new();
    async fn cpu_ids() -> Vec<u64> {
        sysfs::cpu::Device::ids().await
    }
    CPU_IDS.get_or_init(cpu_ids).await.clone()
}

async fn i915_ids() -> Vec<u64> {
    static I915_IDS: OnceCell<Vec<u64>> = OnceCell::const_new();
    async fn i915_ids() -> Vec<u64> {
        sysfs::i915::Device::ids().await
    }
    I915_IDS.get_or_init(i915_ids).await.clone()
}

#[cfg(feature = "nvml")]
async fn nvml_ids() -> Vec<u32> {
    static NVML_IDS: OnceCell<Vec<u32>> = OnceCell::const_new();
    async fn nvml_ids() -> Vec<u32> {
        crate::nvml::Device::ids().await
    }
    NVML_IDS.get_or_init(nvml_ids).await.clone()
}

async fn wait_for_cpu_onoff() {
    const WAIT_FOR_CPU_ONOFF: Duration = Duration::from_millis(300);
    sleep(WAIT_FOR_CPU_ONOFF).await
}

async fn wait_for_cpu_related() {
    const WAIT_FOR_CPU_RELATED: Duration = Duration::from_millis(100);
    sleep(WAIT_FOR_CPU_RELATED).await
}

async fn set_cpus_online(cpu_ids: Vec<u64>) -> Vec<u64> {
    let mut onlined = vec![];
    for cpu_id in cpu_ids {
        if let Ok(online) = sysfs::cpu::online(cpu_id).await {
            if !online && sysfs::cpu::set_online(cpu_id, true).await.is_ok() {
                onlined.push(cpu_id);
            }
        }
    }
    if !onlined.is_empty() {
        wait_for_cpu_onoff().await;
    }
    onlined
}

async fn set_cpus_offline(cpu_ids: Vec<u64>) -> Vec<u64> {
    let mut offlined = vec![];
    for cpu_id in cpu_ids {
        if let Ok(online) = sysfs::cpu::online(cpu_id).await {
            if online && sysfs::cpu::set_online(cpu_id, false).await.is_ok() {
                offlined.push(cpu_id);
            }
        }
    }
    if !offlined.is_empty() {
        wait_for_cpu_onoff().await;
    }
    offlined
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
pub(in crate::cli) enum CardId {
    Index(u64),
    BusId(String),
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, PartialOrd)]
pub(in crate::cli) struct Group {
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

    pub(in crate::cli) pstate_epb: Option<u64>,

    pub(in crate::cli) pstate_epp: Option<String>,

    pub(in crate::cli) rapl_package: Option<u64>,

    pub(in crate::cli) rapl_zone: Option<u64>,

    #[serde(default)]
    #[serde(deserialize_with = "de::power")]
    pub(in crate::cli) rapl_long_limit: Option<Power>,

    #[serde(default)]
    #[serde(deserialize_with = "de::duration")]
    pub(in crate::cli) rapl_long_window: Option<Duration>,

    #[serde(default)]
    #[serde(deserialize_with = "de::power")]
    pub(in crate::cli) rapl_short_limit: Option<Power>,

    #[serde(default)]
    #[serde(deserialize_with = "de::duration")]
    pub(in crate::cli) rapl_short_window: Option<Duration>,
}

impl Group {
    pub(in crate::cli) fn has_cpu_related_values(&self) -> bool {
        self.has_cpu_values() || self.has_cpufreq_values() || self.has_pstate_values()
    }

    pub(in crate::cli) fn has_cpu_values(&self) -> bool {
        self.cpu_on.is_some() || self.cpu_on_each.is_some()
    }

    pub(in crate::cli) fn has_cpufreq_values(&self) -> bool {
        self.cpufreq_gov.is_some() || self.cpufreq_min.is_some() || self.cpufreq_max.is_some()
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

    pub(in crate::cli) fn has_pstate_values(&self) -> bool {
        self.pstate_epb.is_some() || self.pstate_epp.is_some()
    }

    pub(in crate::cli) fn has_rapl_values(&self) -> bool {
        self.rapl_long_limit.is_some()
            || self.rapl_long_window.is_some()
            || self.rapl_short_limit.is_some()
            || self.rapl_short_window.is_some()
    }

    #[allow(clippy::let_and_return)]
    pub(in crate::cli) fn has_values(&self) -> bool {
        let b = self.has_cpu_related_values() || self.has_i915_values() || self.has_rapl_values();
        #[cfg(feature = "nvml")]
        let b = b || self.has_nvml_values();
        b
    }

    pub(in crate::cli) async fn resolve(&mut self) {
        if self.has_cpu_related_values() && self.cpu.is_none() {
            self.cpu = Some(cpu_ids().await);
        }
        if self.has_i915_values() && self.i915.is_none() {
            let i915 = i915_ids().await.into_iter().map(CardId::Index).collect();
            self.i915 = Some(i915);
        }
        #[cfg(feature = "nvml")]
        if self.has_nvml_values() && self.nv.is_none() {
            let nv: Vec<CardId> = nvml_ids()
                .await
                .into_iter()
                .map(|id| CardId::Index(id as u64))
                .collect();
            self.nv = Some(nv);
        }
    }

    pub(in crate::cli) async fn apply_cpu(&self) {
        if let Some(r) = ToPolicy::<sysfs::Cpu>::to_policy(self) {
            r.write().await;
        }
    }

    pub(in crate::cli) async fn apply_cpufreq(&self) {
        if let Some(r) = ToPolicy::<sysfs::Cpufreq>::to_policy(self) {
            r.write().await;
        }
    }

    pub(in crate::cli) async fn apply_drm(&self) {
        if let Some(r) = ToPolicy::<sysfs::I915>::to_policy(self) {
            r.write().await;
        }
    }

    #[cfg(feature = "nvml")]
    pub(in crate::cli) async fn apply_nvml(&self) {
        if let Some(r) = ToPolicy::<nvml::Nvml>::to_policy(self) {
            r.write().await;
        }
    }

    pub(in crate::cli) async fn apply_pstate(&self) {
        if let Some(r) = ToPolicy::<sysfs::IntelPstate>::to_policy(self) {
            r.write().await;
        }
    }

    pub(in crate::cli) async fn apply_rapl(&self) {
        if let Some(r) = ToPolicy::<sysfs::IntelRapl>::to_policy(self) {
            r.write().await;
        }
    }
}

#[derive(Clone, Debug)]
pub(in crate::cli) struct Groups {
    groups: Vec<Group>,
}

impl Groups {
    pub(in crate::cli) fn has_values(&self) -> bool {
        self.groups.iter().any(|k| k.has_values())
    }

    pub(in crate::cli) async fn resolve(&mut self) {
        for k in self.groups.iter_mut() {
            k.resolve().await;
        }
    }

    fn has_cpufreq_values(&self) -> bool {
        self.groups.iter().any(|k| k.has_cpufreq_values())
    }

    fn has_pstate_values(&self) -> bool {
        self.groups.iter().any(|k| k.has_pstate_values())
    }

    fn collect_cpu_ids(&self) -> Vec<u64> {
        let mut ids: Vec<u64> = self
            .groups
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

    pub(in crate::cli) async fn apply(&self) {
        if log::log_enabled!(log::Level::Trace) {
            for (i, k) in self.groups.iter().enumerate() {
                log::trace!("Group {}", i);
                log::trace!("{:#?}", k);
            }
        }
        if self.has_cpufreq_values() || self.has_pstate_values() {
            let onlined = set_cpus_online(self.collect_cpu_ids()).await;

            for (i, g) in self.groups.iter().enumerate() {
                log::debug!("Group {} Pass 0", i);
                g.apply_cpufreq().await;
                g.apply_pstate().await;
                wait_for_cpu_related().await;
            }

            set_cpus_offline(onlined).await;
        }
        for (i, k) in self.groups.iter().enumerate() {
            log::debug!("Group {} Pass 1", i);
            if k.has_cpu_values() {
                k.apply_cpu().await;
                wait_for_cpu_onoff().await;
            }
            k.apply_rapl().await;
            k.apply_drm().await;
            #[cfg(feature = "nvml")]
            k.apply_nvml().await;
        }
    }
}

impl From<Vec<Group>> for Groups {
    fn from(groups: Vec<Group>) -> Self {
        Self { groups }
    }
}

impl From<Group> for Groups {
    fn from(group: Group) -> Self {
        Self::from(vec![group])
    }
}

impl From<Groups> for Vec<Group> {
    fn from(profile: Groups) -> Self {
        profile.groups
    }
}

impl IntoIterator for Groups {
    type IntoIter = std::vec::IntoIter<Group>;
    type Item = Group;

    fn into_iter(self) -> Self::IntoIter {
        self.groups.into_iter()
    }
}
