use zysfs::types::{
    blocking::Write as _,
    devices::system::cpu::{
        Cpu,
        cpufreq::Cpufreq,
        intel_pstate::IntelPstate,
    },
    class::drm::Drm,
};
use crate::cli::Cli;

mod cpu;
mod cpufreq;
mod drm;
mod intel_pstate;
mod nvml;

use nvml::Nvml;

#[derive(Debug, Default)]
pub struct Policy {
    cpu: Option<Cpu>,
    cpufreq: Option<Cpufreq>,
    intel_pstate: Option<IntelPstate>,
    drm: Option<Drm>,
    nvml: Option<Nvml>,
}

impl Policy {
    pub fn apply(&self) {
        if let Some(cpu) = &self.cpu { cpu.write(); }
        if let Some(cpufreq) = &self.cpufreq { cpufreq.write(); }
        if let Some(intel_pstate) = &self.intel_pstate { intel_pstate.write(); }
        if let Some(drm) = &self.drm { drm.write(); }
        if let Some(nvml) = &self.nvml { nvml.write(); }
    }
}

impl From<&Cli> for Policy {
    fn from(cli: &Cli) -> Self {
        Self {
            cpu: cli.into(),
            cpufreq: cli.into(),
            drm: cli.into(),
            intel_pstate: cli.into(),
            nvml: cli.into(),
        }
    }
}
