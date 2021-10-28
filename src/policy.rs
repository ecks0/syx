use zysfs::io::cpu::tokio::{cpu_online, set_cpu_online};
use zysfs::types::{self as sysfs, tokio::Read as _};
use tokio::sync::OnceCell;
use std::time::Duration;

static CPU_IDS: OnceCell<Option<Vec<u64>>> = OnceCell::const_new();

static DRM_IDS: OnceCell<Option<Vec<u64>>> = OnceCell::const_new();

static DRM_I915_IDS: OnceCell<Option<Vec<u64>>> = OnceCell::const_new();

#[cfg(feature = "nvml")]
static NVML_IDS: OnceCell<Option<Vec<u64>>> = OnceCell::const_new();

pub async fn cpu_ids_cached() -> Option<Vec<u64>> {
    async fn ids() -> Option<Vec<u64>> { sysfs::cpu::Policy::ids().await }
    CPU_IDS.get_or_init(ids).await.clone()
}

pub async fn drm_ids_cached() -> Option<Vec<u64>> {
    async fn ids() -> Option<Vec<u64>> { sysfs::drm::Card::ids().await }
    DRM_IDS.get_or_init(ids).await.clone()
}

pub async fn drm_ids_i915_cached() -> Option<Vec<u64>> {
    async fn ids() -> Option<Vec<u64>> {
        use zysfs::io::drm::tokio::driver;
        let mut ids = vec![];
        if let Some(drm_ids) = drm_ids_cached().await {
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

#[cfg(feature = "nvml")]
pub async fn nvml_ids_cached() -> Option<Vec<u64>> {
    async fn ids() -> Option<Vec<u64>> { nvml_facade::Nvml::ids().map(|ids| ids.into_iter().map(u64::from).collect()) }
    NVML_IDS.get_or_init(ids).await.clone()
}

const CPU_ONOFF_MILLIS: u64 = 200;

pub async fn set_all_cpus_online() -> Vec<u64> {
    let cpu_ids = cpu_ids_cached().await.unwrap_or_else(Vec::new);
    let mut onlined = vec![];
    for cpu_id in cpu_ids {
        if let Ok(online) = cpu_online(cpu_id).await {
            if !online && set_cpu_online(cpu_id, true).await.is_ok() {
                onlined.push(cpu_id);
            }
        }
    }
    if !onlined.is_empty() {
        tokio::time::sleep(Duration::from_millis(CPU_ONOFF_MILLIS)).await;
    }
    onlined
}

pub async fn set_cpus_offline(cpu_ids: Vec<u64>) {
    if !cpu_ids.is_empty() {
        for id in cpu_ids {
            let _ = set_cpu_online(id, false).await;
        }
        tokio::time::sleep(Duration::from_millis(CPU_ONOFF_MILLIS)).await;
    }
}

impl From<&crate::Knobs> for Option<sysfs::cpu::Cpu> {
    fn from(k: &crate::Knobs) -> Self {
        if !k.has_cpu_values() { return None; }
        let cpu_ids = k.cpu.clone()?;
        if cpu_ids.is_empty() { return None; }
        let mut policies = vec![];
        if let Some(cpu_on) = k.cpu_on {
            for cpu_id in cpu_ids {
                let policy = sysfs::cpu::Policy {
                    id: Some(cpu_id),
                    cpu_online: Some(cpu_on)
                };
                policies.push(policy);
            }
        }
        if let Some(cpus_on) = k.cpus_on.clone() {
            for (cpu_id, cpu_on) in cpus_on {
                if let Some(policy) = policies
                    .iter_mut()
                    .find(|p| Some(cpu_id) == p.id)
                {
                    policy.cpu_online = Some(cpu_on);
                }
                else
                {
                    let policy = sysfs::cpu::Policy {
                        id: Some(cpu_id),
                        cpu_online: Some(cpu_on),
                    };
                    policies.push(policy);
                }
            }
        }
        policies.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        let s = sysfs::cpu::Cpu {
            policies: Some(policies),
        };
        Some(s)
    }
}

impl From<&crate::Knobs> for Option<sysfs::cpufreq::Cpufreq> {
    fn from(k: &crate::Knobs) -> Self {
        if !k.has_cpufreq_values() { return None; }
        let policy_ids = k.cpu.clone()?;
        if policy_ids.is_empty() { return None; }
        let scaling_min_freq = k.cpufreq_min.map(|f| f.as_kilohertz().ceil() as u64);
        let scaling_max_freq = k.cpufreq_max.map(|f| f.as_kilohertz().ceil() as u64);
        let mut policies = vec![];
        for policy_id in policy_ids {
            let policy = sysfs::cpufreq::Policy {
                id: Some(policy_id),
                scaling_governor: k.cpufreq_gov.clone(),
                scaling_min_freq,
                scaling_max_freq,
                ..Default::default()
            };
            policies.push(policy);
        }
        let s = sysfs::cpufreq::Cpufreq {
            policies: Some(policies),
            ..Default::default()
        };
        Some(s)
    }
}

impl From<&crate::Knobs> for Option<sysfs::drm::Drm> {
    fn from(k: &crate::Knobs) -> Self {
        if !k.has_drm_values() { return None; }
        let mut cards = vec![];
        if k.has_drm_i915_values() {
            if let Some(i915_ids) = k.drm_i915.clone() {
                if !i915_ids.is_empty() {
                    let driver_policy = sysfs::drm::DriverPolicy::I915(
                        sysfs::drm::I915 {
                            min_freq_mhz: k.drm_i915_min.map(|f| f.as_megahertz().ceil() as u64),
                            max_freq_mhz: k.drm_i915_max.map(|f| f.as_megahertz().ceil() as u64),
                            boost_freq_mhz: k.drm_i915_boost.map(|f| f.as_megahertz() as u64),
                            ..Default::default()
                        }
                    );
                    for card_id in i915_ids {
                        let card_id = match card_id {
                            crate::CardId::Id(id) => id,
                            crate::CardId::PciId(_) => panic!("PCI ID support is not yet implemente for drm-i915"),
                        };
                        let card = sysfs::drm::Card {
                            id: Some(card_id),
                            driver_policy: Some(driver_policy.clone()),
                            ..Default::default()
                        };
                        cards.push(card);
                    }
                }
            }
        }
        let s = sysfs::drm::Drm {
            cards: Some(cards),
        };
        Some(s)
    }
}

#[cfg(feature = "nvml")]
#[derive(Clone, Debug, Default)]
pub struct NvmlPolicy {
    id: Option<u32>,
    gpu_clock: Option<(u32, u32)>,
    gpu_clock_reset: Option<()>,
    power_limit: Option<u32>,
}

#[cfg(feature = "nvml")]
impl NvmlPolicy {
    pub fn write(&self) {
        let id = if let Some(id) = self.id { id } else { return; };
        let device = if let Some(d) = nvml_facade::Nvml::device_for_id(id) { d } else { return; };
        if let Some((min, max)) = self.gpu_clock {
            device.clocks().set_gpu_locked_clocks(min, max);
        }
        if self.gpu_clock_reset.is_some() {
            device.clocks().reset_gpu_locked_clocks();
        }
        if let Some(max) = self.power_limit {
            device.power().set_limit(max);
        }
    }
}

#[cfg(feature = "nvml")]
#[derive(Clone, Debug, Default)]
pub struct NvmlPolicies {
    policies: Option<Vec<NvmlPolicy>>,
}

#[cfg(feature = "nvml")]
impl NvmlPolicies {
    pub fn write(&self) {
        if let Some(pols) = &self.policies {
            for pol in pols {
                pol.write();
            }
        }
    }
}

#[cfg(feature = "nvml")]
fn nvml_card_ids(ids: Vec<crate::CardId>) -> Option<Vec<u32>> {
    fn card_id(id: crate::CardId) -> Option<u32> {
        match id {
            crate::CardId::Id(id) => Some(id as u32),
            crate::CardId::PciId(id) => Some(nvml_facade::Nvml::device_for_pci_id(&id)?.card().id()?),
        }
    }
    let mut indices = vec![];
    for id in ids {
        match card_id(id) {
            Some(id) => indices.push(id),
            _ => continue,
        }
    }
    if indices.is_empty() { None } else {
        indices.sort_unstable();
        indices.dedup();
        Some(indices)
    }
}

#[cfg(feature = "nvml")]
impl From<&crate::Knobs> for Option<NvmlPolicies> {
    fn from(k: &crate::Knobs) -> Self {
        if !k.has_nvml_values() { return None; }
        let ids = k.nvml.clone()
            .and_then(nvml_card_ids)
            .or_else(nvml_facade::Nvml::ids)?;
        if ids.is_empty() { return None; }
        let gpu_clock = k.nvml_gpu_min
            .and_then(|min|
                k.nvml_gpu_max
                    .map(|max|
                        (min.as_megahertz() as u32, max.as_megahertz().ceil() as u32)));
        let power_limit = k
            .nvml_power_limit
            .map(|p| p.as_milliwatts().ceil() as u32);
        let mut policies = vec![];
        for id in ids {
            let pol = NvmlPolicy {
                id: Some(id),
                gpu_clock,
                gpu_clock_reset: k.nvml_gpu_reset.and_then(|v| if v { Some(()) } else { None }),
                power_limit,
            };
            policies.push(pol);
        }
        let s = NvmlPolicies {
            policies: Some(policies),
        };
        Some(s)
    }
}

impl From<&crate::Knobs> for Option<sysfs::intel_pstate::IntelPstate> {
    fn from(k: &crate::Knobs) -> Self {
        if !k.has_pstate_values() { return None; }
        let policy_ids = k.cpu.clone()?;
        if policy_ids.is_empty() { return None; }
        let mut policies = vec![];
        for policy_id in policy_ids {
            let policy = sysfs::intel_pstate::Policy {
                id: Some(policy_id),
                energy_perf_bias: k.pstate_epb,
                energy_performance_preference: k.pstate_epp.clone(),
                ..Default::default()
            };
            policies.push(policy);
        }
        let s = sysfs::intel_pstate::IntelPstate {
            policies: Some(policies),
            ..Default::default()
        };
        Some(s)
    }
}

impl From<&crate::Knobs> for Option<sysfs::intel_rapl::IntelRapl> {
    fn from(k: &crate::Knobs) -> Self {
        if !k.has_rapl_values() { return None; }
        let id = sysfs::intel_rapl::ZoneId { zone: k.rapl_package?, subzone: k.rapl_zone };
        let constraints: Vec<sysfs::intel_rapl::Constraint> =
            [
                (0, k.rapl_c0_limit, k.rapl_c0_window),
                (1, k.rapl_c1_limit, k.rapl_c1_window),
            ]
            .iter()
            .filter_map(|(id, limit, window)|
                if limit.is_some() || window.is_some() {
                    let c = sysfs::intel_rapl::Constraint {
                        id: Some(*id),
                        power_limit_uw: limit.map(|v| v.as_microwatts().ceil() as u64),
                        time_window_us: window.map(|v| v.as_micros().try_into().unwrap()),
                        ..Default::default()
                    };
                    Some(c)
                } else {
                    None
                }
            )
            .collect();
        let s = sysfs::intel_rapl::IntelRapl {
            policies: Some(vec![
                sysfs::intel_rapl::Policy {
                    id: Some(id),
                    constraints: Some(constraints),
                    ..Default::default()
                },
            ]),
        };
        Some(s)
    }
}
