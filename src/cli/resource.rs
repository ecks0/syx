use crate::cli::group::{CardId, Group};
#[cfg(feature = "nvml")]
use crate::nvml;
use crate::{sysfs, Resource};

pub trait ToResource<R>
where
    R: Resource,
{
    fn to_resource(&self) -> Option<R>;
}

impl ToResource<sysfs::Cpu> for Group {
    fn to_resource(&self) -> Option<sysfs::Cpu> {
        if !self.has_cpu_values() {
            return None;
        }
        let mut devices: Vec<sysfs::cpu::Device> = self
            .cpu
            .clone()
            .map(|ids| {
                ids.into_iter()
                    .map(|id| {
                        let online = self.cpu_on;
                        sysfs::cpu::Device { id, online }
                    })
                    .collect()
            })
            .unwrap_or_default();
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
        if devices.is_empty() {
            None
        } else {
            devices.sort_unstable_by(|a, b| a.id.cmp(&b.id));
            let r = sysfs::Cpu { devices };
            Some(r)
        }
    }
}

impl ToResource<sysfs::Cpufreq> for Group {
    fn to_resource(&self) -> Option<sysfs::Cpufreq> {
        if !self.has_cpufreq_values() {
            return None;
        }
        let scaling_min_freq = self.cpufreq_min.map(|f| f.as_kilohertz().round() as u64);
        let scaling_max_freq = self.cpufreq_max.map(|f| f.as_kilohertz().round() as u64);
        let policies: Vec<sysfs::cpufreq::Policy> = self
            .cpu
            .clone()
            .map(|ids| {
                ids.into_iter()
                    .map(|id| {
                        let scaling_governor = self.cpufreq_gov.clone();
                        sysfs::cpufreq::Policy {
                            id,
                            scaling_governor,
                            scaling_min_freq,
                            scaling_max_freq,
                            ..Default::default()
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();
        if policies.is_empty() {
            None
        } else {
            let r = sysfs::Cpufreq { policies };
            Some(r)
        }
    }
}

impl ToResource<sysfs::I915> for Group {
    fn to_resource(&self) -> Option<sysfs::I915> {
        if !self.has_i915_values() {
            return None;
        }
        let min_freq_mhz = self.i915_min.map(|f| f.as_megahertz().round() as u64);
        let max_freq_mhz = self.i915_max.map(|f| f.as_megahertz().round() as u64);
        let boost_freq_mhz = self.i915_boost.map(|f| f.as_megahertz().round() as u64);
        let devices: Vec<sysfs::i915::Device> = self
            .i915
            .clone()
            .map(|ids| {
                ids.into_iter()
                    .map(|id| {
                        let id = match id {
                            CardId::Index(id) => id,
                            CardId::BusId(_) => {
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
                    .collect()
            })
            .unwrap_or_default();
        if devices.is_empty() {
            None
        } else {
            let r = sysfs::I915 { devices };
            Some(r)
        }
    }
}

impl ToResource<sysfs::IntelPstate> for Group {
    fn to_resource(&self) -> Option<sysfs::IntelPstate> {
        if !self.has_pstate_values() {
            return None;
        }
        let energy_perf_bias = self.pstate_epb;
        let policies: Vec<sysfs::intel_pstate::Policy> = self
            .cpu
            .clone()
            .map(|ids| {
                ids.into_iter()
                    .map(|id| {
                        let energy_performance_preference = self.pstate_epp.clone();
                        sysfs::intel_pstate::Policy {
                            id,
                            energy_perf_bias,
                            energy_performance_preference,
                            ..Default::default()
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();
        if policies.is_empty() {
            None
        } else {
            let s = sysfs::intel_pstate::IntelPstate {
                policies,
                ..Default::default()
            };
            Some(s)
        }
    }
}

impl ToResource<sysfs::IntelRapl> for Group {
    fn to_resource(&self) -> Option<sysfs::IntelRapl> {
        if !self.has_rapl_values() {
            return None;
        }
        let id = sysfs::intel_rapl::ZoneId {
            zone: self.rapl_package?,
            subzone: self.rapl_zone,
        };
        let constraints: Vec<sysfs::intel_rapl::Constraint> = [
            ("long_term", self.rapl_long_limit, self.rapl_long_window),
            ("short_term", self.rapl_short_limit, self.rapl_short_window),
        ]
        .iter()
        .filter_map(|(name, limit, window)| {
            if limit.is_some() || window.is_some() {
                let name = Some(name.to_string());
                let power_limit_uw = limit.map(|v| v.as_microwatts().round() as u64);
                let time_window_us = window.map(|v| v.as_micros().try_into().unwrap());
                let c = sysfs::intel_rapl::Constraint {
                    name,
                    power_limit_uw,
                    time_window_us,
                    ..Default::default()
                };
                Some(c)
            } else {
                None
            }
        })
        .collect();
        let devices = vec![sysfs::intel_rapl::Device {
            id,
            constraints,
            ..Default::default()
        }];
        let r = sysfs::IntelRapl { devices };
        Some(r)
    }
}

#[cfg(feature = "nvml")]
impl ToResource<nvml::Nvml> for Group {
    fn to_resource(&self) -> Option<nvml::Nvml> {
        if !self.has_nvml_values() {
            return None;
        }
        let gfx_freq_min = self.nv_gpu_min.map(|f| f.as_megahertz().round() as u32);
        let gfx_freq_max = self.nv_gpu_max.map(|f| f.as_megahertz().round() as u32);
        let gfx_freq_reset = self.nv_gpu_reset;
        let power_limit = self
            .nv_power_limit
            .map(|p| p.as_milliwatts().round() as u32);
        let devices: Vec<nvml::Device> = self
            .nv
            .clone()
            .map(|ids| {
                ids.into_iter()
                    .map(|id| {
                        let id = match id {
                            CardId::Index(id) => id.try_into().unwrap(),
                            CardId::BusId(_) => {
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
                    .collect()
            })
            .unwrap_or_default();
        if devices.is_empty() {
            None
        } else {
            let r = nvml::Nvml { devices };
            Some(r)
        }
    }
}
