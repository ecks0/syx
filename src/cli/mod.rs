use measurements::{Frequency, Power};
use nvml_facade::Nvml;
use zysfs::io::devices::system::cpu::blocking::cpus;
use zysfs::io::class::drm::blocking::{cards as drm_cards, driver as drm_driver};
use crate::{Result, policy::Policy, format};

mod clap;
mod logging;
mod parse;

#[derive(Debug)]
pub struct Cli {
    pub show_cpu: Option<()>,
    pub show_intel_pstate: Option<()>,
    pub show_drm: Option<()>,
    pub show_nvml: Option<()>,
    pub cpu: Option<Vec<u64>>,
    pub cpu_on: Option<bool>,
    pub cpu_on_each: Option<Vec<(u64, bool)>>,
    pub cpufreq_gov: Option<String>,
    pub cpufreq_min: Option<Frequency>,
    pub cpufreq_max: Option<Frequency>,
    pub pstate_epb: Option<u64>,
    pub pstate_epp: Option<String>,
    pub drm_i915: Option<Vec<u64>>,
    pub drm_i915_min: Option<Frequency>,
    pub drm_i915_max: Option<Frequency>,
    pub drm_i915_boost: Option<Frequency>,
    pub nvml: Option<Vec<u32>>,
    pub nvml_gpu_clock: Option<(Frequency, Frequency)>,
    pub nvml_gpu_clock_reset: Option<()>,
    pub nvml_power_limit: Option<Power>,
}

impl Cli {
    pub fn from_args(argv: &[String]) -> Result<Self> {
        clap::parse(argv)
    }

    pub fn has_show_args(&self) -> bool {
        self.show_cpu.is_some() ||
        self.show_intel_pstate.is_some() ||
        self.show_drm.is_some() ||
        self.show_nvml.is_some()
    }

    pub fn has_cpu_args(&self) -> bool {
        self.cpu_on.is_some() ||
        self.cpu_on_each.is_some()
    }

    pub fn has_cpufreq_args(&self) -> bool {
        self.cpufreq_gov.is_some() ||
        self.cpufreq_min.is_some() ||
        self.cpufreq_max.is_some()
    }

    pub fn has_intel_pstate_args(&self) -> bool {
        self.pstate_epb.is_some() ||
        self.pstate_epp.is_some()
    }

    pub fn has_cpu_or_related_args(&self) -> bool {
        self.has_cpu_args() ||
        self.has_cpufreq_args() ||
        self.has_intel_pstate_args()
    }

    pub fn has_drm_i915_args(&self) -> bool {
        self.drm_i915_min.is_some() ||
        self.drm_i915_max.is_some() ||
        self.drm_i915_boost.is_some()
    }

    pub fn has_drm_args(&self) -> bool {
        self.has_drm_i915_args()
    }

    pub fn has_nvml_args(&self) -> bool {
        self.nvml_gpu_clock.is_some() ||
        self.nvml_gpu_clock_reset.is_some() ||
        self.nvml_power_limit.is_some()
    }

    pub fn cpu(&self) -> Option<Vec<u64>> {
        let cpu =
            if let Some(cpu) = &self.cpu {
                let mut cpu = cpu.clone();
                cpu.sort_unstable();
                cpu.dedup();
                cpu
            } else {
                cpus().ok()?
            };
        if cpu.is_empty() { None } else { Some(cpu) }
    }

    fn drm(&self, arg_value: &Option<Vec<u64>>, driver: &str) -> Option<Vec<u64>> {
        let card_ids =
            if let Some(card_ids) = arg_value {
                let mut card_ids = card_ids.clone();
                card_ids.sort_unstable();
                card_ids.dedup();
                card_ids
            } else {
                drm_cards()
                    .ok()?
                    .into_iter()
                    .filter(|id| {
                        if let Ok(drv) = drm_driver(*id) {
                            if drv == driver {
                                return true;
                            }
                        }
                        false
                    })
                    .collect()
            };
        if card_ids.is_empty() { None } else { Some(card_ids) }
    }

    pub fn drm_i915(&self) -> Option<Vec<u64>> {
        self.drm(&self.drm_i915, "i915")
    }

    pub fn nvml(&self) -> Option<Vec<u32>> {
        if let Some(card_ids) = self.nvml.clone() {
            let mut card_ids = card_ids;
            card_ids.sort_unstable();
            card_ids.dedup();
            Some(card_ids)
        } else {
            Nvml::ids()
        }
    }

    pub fn apply(&self) {
        let policy: Policy = self.into();
        policy.apply();
    }

    pub fn show(&self) {
        let mut s = vec![String::with_capacity(0)];
        let show_all = !self.has_show_args();
        if show_all || self.show_cpu.is_some() {
            if let Some(ss) = format::format_cpu() { s.push(ss); }
        }
        if show_all || self.show_intel_pstate.is_some() {
            if let Some(ss) = format::format_intel_pstate() { s.push(ss); }
        }
        if show_all || self.show_drm.is_some() {
            if let Some(ss) = format::format_drm() { s.push(ss); }
        }
        if show_all || self.show_nvml.is_some() {
            if let Some(ss) = format::format_nvml() { s.push(ss); }
        }
        if !s.is_empty() { println!("{}", s.join("\n")); }
    }
}
