use zysfs::types::{
    cpu::Cpu,
    cpufreq::Cpufreq,
    drm::Drm,
    intel_pstate::IntelPstate,
};
use zysfs::types::std::Write as _;
use crate::cli::Cli;

mod cpu;
mod cpufreq;
mod drm;
mod intel_pstate;
#[cfg(feature = "nvml")]
mod nvml;

#[cfg(feature = "nvml")]
use nvml::Nvml;

#[derive(Debug, Default)]
pub struct Policy {
    cpu: Option<Cpu>,
    cpufreq: Option<Cpufreq>,
    intel_pstate: Option<IntelPstate>,
    drm: Option<Drm>,
    #[cfg(feature = "nvml")]
    nvml: Option<Nvml>,
}

impl Policy {
    pub fn apply(&self) {
        if self.cpufreq.is_some() || self.intel_pstate.is_some() {
            let onlined_cpus = cpu::set_all_cpus_online();
            if let Some(cpufreq) = &self.cpufreq { cpufreq.write(); }
            if let Some(intel_pstate) = &self.intel_pstate { intel_pstate.write(); }
            if let Some(ids) = onlined_cpus { cpu::set_cpus_offline(ids); }
        }
        if let Some(cpu) = &self.cpu { cpu.write(); }
        if let Some(drm) = &self.drm { drm.write(); }
        #[cfg(feature = "nvml")]
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
            #[cfg(feature = "nvml")]
            nvml: cli.into(),
        }
    }
}
