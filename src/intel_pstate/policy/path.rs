use std::path::PathBuf;

use crate::cpu::path::cpu_attr;
use crate::cpufreq::path::policy_attr;

pub(crate) fn energy_perf_bias(id: u64) -> PathBuf {
    let mut p = cpu_attr(id, "power");
    p.push("energy_perf_bias");
    p
}

pub(crate) fn energy_performance_preference(id: u64) -> PathBuf {
    policy_attr(id, "energy_performance_preference")
}

pub(crate) fn energy_performance_available_preferences(id: u64) -> PathBuf {
    policy_attr(id, "energy_performance_available_preferences")
}
