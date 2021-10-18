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

#[derive(Debug, Default)]
pub struct Policy {
    cpu: Option<Cpu>,
    cpufreq: Option<Cpufreq>,
    intel_pstate: Option<IntelPstate>,
    drm: Option<Drm>,
}

impl Policy {
    pub fn apply(&self) {
        if let Some(cpu) = &self.cpu { cpu.write(); }
        if let Some(cpufreq) = &self.cpufreq { cpufreq.write(); }
        if let Some(intel_pstate) = &self.intel_pstate { intel_pstate.write(); }
        if let Some(drm) = &self.drm { drm.write(); }
    }
}

impl From<&Cli> for Policy {
    fn from(cli: &Cli) -> Self {
        let mut s = Self::default();
        if cli.has_cpu_or_related_args() {
            if let Some(cpu_ids) = cli.cpus() {
                s.cpu = cpu::policy(cli, &cpu_ids);
                s.cpufreq = cpufreq::policy(cli, &cpu_ids);
                s.intel_pstate = intel_pstate::policy(cli, &cpu_ids);
            }
        }
        s.drm = drm::policy(cli);
        s
    }
}
