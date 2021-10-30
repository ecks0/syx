use zysfs::io::cpu::tokio::{cpu_online, set_cpu_online};
use zysfs::types as sysfs;

pub async fn set_cpus_online(cpu_ids: Vec<u64>) -> Vec<u64> {
    let mut onlined = vec![];
    for cpu_id in cpu_ids {
        if let Ok(online) = cpu_online(cpu_id).await {
            if !online && set_cpu_online(cpu_id, true).await.is_ok() {
                onlined.push(cpu_id);
            }
        }
    }
    onlined
}

pub async fn set_cpus_offline(cpu_ids: Vec<u64>) -> Vec<u64> {
    let mut offlined = vec![];
    for cpu_id in cpu_ids {
        if let Ok(online) = cpu_online(cpu_id).await {
            if online && set_cpu_online(cpu_id, false).await.is_ok() {
                offlined.push(cpu_id);
            }
        }
    }
    offlined
}

impl From<&crate::Knobs> for Option<sysfs::cpu::Cpu> {
    fn from(k: &crate::Knobs) -> Self {
        if !k.has_cpu_values() { return None; }
        let policies: Option<Vec<sysfs::cpu::Policy>> =
            k.cpu.clone().map(|ids| ids
                .into_iter()
                .map(|id|
                    sysfs::cpu::Policy {
                        id: Some(id),
                        cpu_online: k.cpu_online,
                    })
                .collect());
        if policies.as_ref().map(|p| p.is_empty()).unwrap_or(true) { return None; }
        let s = sysfs::cpu::Cpu {
            policies,
        };
        Some(s)
    }
}

impl From<&crate::Knobs> for Option<sysfs::cpufreq::Cpufreq> {
    fn from(k: &crate::Knobs) -> Self {
        if !k.has_cpufreq_values() { return None; }
        let scaling_min_freq = k.cpufreq_min.map(|f| f.as_kilohertz().round() as u64);
        let scaling_max_freq = k.cpufreq_max.map(|f| f.as_kilohertz().round() as u64);
        let policies: Option<Vec<sysfs::cpufreq::Policy>> =
            k.cpu.clone().map(|ids| ids
                .into_iter()
                .map(|id|
                    sysfs::cpufreq::Policy {
                        id: Some(id),
                        scaling_governor: k.cpufreq_gov.clone(),
                        scaling_min_freq,
                        scaling_max_freq,
                        ..Default::default()
                    })
                .collect());
        if policies.as_ref().map(|p| p.is_empty()).unwrap_or(true) { return None; }
        let s = sysfs::cpufreq::Cpufreq {
            policies,
            ..Default::default()
        };
        Some(s)
    }
}

impl From<&crate::Knobs> for Option<sysfs::drm::Drm> {
    fn from(k: &crate::Knobs) -> Self {
        if !k.has_drm_values() { return None; }
        let cards = vec![

            match k.has_drm_i915_values() {
                true => {
                    let min_freq_mhz = k.drm_i915_min.map(|f| f.as_megahertz().round() as u64);
                    let max_freq_mhz = k.drm_i915_max.map(|f| f.as_megahertz().round() as u64);
                    let boost_freq_mhz = k.drm_i915_boost.map(|f| f.as_megahertz().round() as u64);
                    k.drm_i915.clone().map(|ids| ids
                        .into_iter()
                        .map(|id|
                            sysfs::drm::Card {
                                id: match id {
                                    crate::CardId::Id(id) => Some(id),
                                    crate::CardId::PciId(_) => panic!("Indexing drm-i915 cards by PCI ID is not yet implemented"),
                                },
                                driver_policy: Some(
                                    sysfs::drm::DriverPolicy::I915(
                                        sysfs::drm::I915 {
                                            min_freq_mhz,
                                            max_freq_mhz,
                                            boost_freq_mhz,
                                            ..Default::default()
                                        },
                                    )),
                                ..Default::default()
                            })
                        .collect())
                    .unwrap_or_else(Vec::new)
                },
                false => vec![],
            },

            // ... insert amd gpu support here ...
        ];
        let cards: Vec<sysfs::drm::Card> = cards.into_iter().flatten().collect();
        if cards.is_empty() { return None; }
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
impl From<&crate::Knobs> for Option<NvmlPolicies> {
    fn from(k: &crate::Knobs) -> Self {
        if !k.has_nvml_values() { return None; }
        let gpu_clock = k.nvml_gpu_min
            .and_then(|min|
                k.nvml_gpu_max
                    .map(|max|
                        (min.as_megahertz().round() as u32, max.as_megahertz().round() as u32)));
        let gpu_clock_reset = k.nvml_gpu_reset.and_then(|v| if v { Some(()) } else { None });
        let power_limit = k.nvml_power_limit.map(|p| p.as_milliwatts().round() as u32);
        let policies: Option<Vec<NvmlPolicy>> =
            k.nvml.clone().map(|ids| ids
                .into_iter()
                .map(|id|
                    NvmlPolicy {
                        id: match id {
                            crate::CardId::Id(id) => Some(id.try_into().unwrap()),
                            crate::CardId::PciId(id) => nvml_facade::Nvml::device_for_pci_id(&id).and_then(|d| d.card().id()),
                        },
                        gpu_clock,
                        gpu_clock_reset,
                        power_limit,
                    })
                .collect());
        if policies.as_ref().map(|p| p.is_empty()).unwrap_or(true) { return None; }
        let s = NvmlPolicies {
            policies,
        };
        Some(s)
    }
}

impl From<&crate::Knobs> for Option<sysfs::intel_pstate::IntelPstate> {
    fn from(k: &crate::Knobs) -> Self {
        if !k.has_pstate_values() { return None; }
        let policies: Option<Vec<sysfs::intel_pstate::Policy>> =
            k.cpu.clone().map(|ids| ids
                .into_iter()
                .map(|id|
                    sysfs::intel_pstate::Policy {
                        id: Some(id),
                        energy_perf_bias: k.pstate_epb,
                        energy_performance_preference: k.pstate_epp.clone(),
                        ..Default::default()
                    })
                .collect());
        if policies.as_ref().map(|p| p.is_empty()).unwrap_or(true) { return None; }
        let s = sysfs::intel_pstate::IntelPstate {
            policies,
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
                ("long_term", k.rapl_long_limit, k.rapl_long_window),
                ("short_term", k.rapl_short_limit, k.rapl_short_window),
            ]
            .iter()
            .filter_map(|(name, limit, window)|
                if limit.is_some() || window.is_some() {
                    let c = sysfs::intel_rapl::Constraint {
                        name: Some(name.to_string()),
                        power_limit_uw: limit.map(|v| v.as_microwatts().round() as u64),
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
