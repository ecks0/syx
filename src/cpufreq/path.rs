use std::path::PathBuf;

pub(crate) fn root() -> PathBuf {
    PathBuf::from("/sys/devices/system/cpu/cpufreq")
}

pub(crate) fn policy(id: u64) -> PathBuf {
    let mut p = root();
    p.push(&format!("policy{}", id));
    p
}

pub(crate) fn policy_attr(i: u64, a: &str) -> PathBuf {
    let mut p = policy(i);
    p.push(a);
    p
}

pub(crate) fn cpuinfo_max_freq(id: u64) -> PathBuf {
    policy_attr(id, "cpuinfo_max_freq")
}

pub(crate) fn cpuinfo_min_freq(id: u64) -> PathBuf {
    policy_attr(id, "cpuinfo_min_freq")
}

pub(crate) fn scaling_cur_freq(id: u64) -> PathBuf {
    policy_attr(id, "scaling_cur_freq")
}

pub(crate) fn scaling_driver(id: u64) -> PathBuf {
    policy_attr(id, "scaling_driver")
}

pub(crate) fn scaling_governor(id: u64) -> PathBuf {
    policy_attr(id, "scaling_governor")
}

pub(crate) fn scaling_available_governors(id: u64) -> PathBuf {
    policy_attr(id, "scaling_available_governors")
}

pub(crate) fn scaling_max_freq(id: u64) -> PathBuf {
    policy_attr(id, "scaling_max_freq")
}

pub(crate) fn scaling_min_freq(id: u64) -> PathBuf {
    policy_attr(id, "scaling_min_freq")
}
