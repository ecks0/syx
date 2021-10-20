use measurements::{Frequency, Power};
use crate::{format, policy::Policy};

mod clap;
mod logging;
mod parse;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error: {flag} {msg}")]
    Parse {
        flag: &'static str,
        msg: &'static str,
    },
}

impl Error {
    fn parse(flag: &'static str, msg: &'static str) -> Self { Self::Parse { flag, msg } }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub enum CardId {
    Index(u64),
    PciId(String),
}

#[derive(Debug)]
pub struct Cli {
    pub quiet: Option<()>,
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
    pub drm_i915: Option<Vec<CardId>>,
    pub drm_i915_min: Option<Frequency>,
    pub drm_i915_max: Option<Frequency>,
    pub drm_i915_boost: Option<Frequency>,
    pub nvml: Option<Vec<CardId>>,
    pub nvml_gpu_freq: Option<(Frequency, Frequency)>,
    pub nvml_gpu_freq_reset: Option<()>,
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

    pub fn has_drm_i915_args(&self) -> bool {
        self.drm_i915_min.is_some() ||
        self.drm_i915_max.is_some() ||
        self.drm_i915_boost.is_some()
    }

    pub fn has_drm_args(&self) -> bool {
        self.has_drm_i915_args()
    }

    pub fn has_nvml_args(&self) -> bool {
        self.nvml_gpu_freq.is_some() ||
        self.nvml_gpu_freq_reset.is_some() ||
        self.nvml_power_limit.is_some()
    }

    fn apply(&self) {
        let policy: Policy = self.into();
        policy.apply();
    }

    fn show(&self) {
        let mut s = vec![String::new()];
        let show_all = !self.has_show_args();
        if show_all || self.show_cpu.is_some() {
            if let Some(ss) = format::cpu() { s.push(ss); }
        }
        if show_all || self.show_intel_pstate.is_some() {
            if let Some(ss) = format::intel_pstate() { s.push(ss); }
        }
        if show_all || self.show_drm.is_some() {
            if let Some(ss) = format::drm() { s.push(ss); }
        }
        if show_all || self.show_nvml.is_some() {
            if let Some(ss) = format::nvml() { s.push(ss); }
        }
        if !s.is_empty() { println!("{}", s.join("\n")); }
    }

    pub fn run(&self) {
        self.apply();
        if self.quiet.is_none() {
            self.show();
        }
    }
}
