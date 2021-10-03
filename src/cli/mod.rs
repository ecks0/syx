use measurements::frequency::Frequency;
use zysfs::io::devices::system::cpu::blocking::cpus;
use zysfs::io::class::drm::blocking::{cards as drm_cards, driver as drm_driver};
use crate::Result;

mod app;
mod logging;
mod parse;

#[derive(Debug)]
pub struct Cli {
    pub cpus: Option<Vec<u64>>,
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
}

impl Cli {
    pub fn from_args(argv: &[String]) -> Result<Self> {
        let argv0 = argv
            .first()
            .map(|s| s.as_str())
            .unwrap_or("knobs")
            .split('/')
            .last()
            .unwrap_or("knobs");
        let m = app::build(argv0).get_matches_from(argv);
        if m.is_present("verbose") { logging::enable()?; }
        Ok(Self {
            cpus: parse::cpus(m.value_of("cpus"))?,
            cpu_on: parse::cpu_on(m.value_of("cpu-on"))?,
            cpu_on_each: parse::cpu_on_each(m.value_of("cpu-on-each"))?,
            cpufreq_gov: parse::cpufreq_gov(m.value_of("cpufreq-gov")),
            cpufreq_min: parse::cpufreq_min(m.value_of("cpufreq-min"))?,
            cpufreq_max: parse::cpufreq_max(m.value_of("cpufreq-max"))?,
            pstate_epb: parse::pstate_epb(m.value_of("pstate-epb"))?,
            pstate_epp: parse::pstate_epp(m.value_of("pstate-epp")),
            drm_i915: parse::drm_i915(m.value_of("drm-i915"))?,
            drm_i915_min: parse::drm_i915_min(m.value_of("drm-i915-min"))?,
            drm_i915_max: parse::drm_i915_max(m.value_of("drm-i915-max"))?,
            drm_i915_boost: parse::drm_i915_boost(m.value_of("drm-i915-boost"))?,
        })
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

    pub fn cpus(&self) -> Option<Vec<u64>> {
        let cpus =
            if let Some(cpus) = &self.cpus {
                let mut cpus = cpus.clone();
                cpus.sort_unstable();
                cpus.dedup();
                cpus
            } else {
                cpus().ok()?
            };
        if cpus.is_empty() { None } else { Some(cpus) }
    }

    fn drm_cards(&self, arg_value: &Option<Vec<u64>>, driver: &str) -> Option<Vec<u64>> {
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
        self.drm_cards(&self.drm_i915, "i915")
    }
}
