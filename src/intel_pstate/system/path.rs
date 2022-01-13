use std::path::PathBuf;

pub(crate) fn root() -> PathBuf {
    PathBuf::from("/sys/devices/system/cpu/intel_pstate")
}

pub(crate) fn root_attr(a: &str) -> PathBuf {
    root().join(a)
}

pub(crate) fn max_perf_pct() -> PathBuf {
    root_attr("max_perf_pct")
}

pub(crate) fn min_perf_pct() -> PathBuf {
    root_attr("min_perf_pct")
}

pub(crate) fn no_turbo() -> PathBuf {
    root_attr("no_turbo")
}

pub(crate) fn status() -> PathBuf {
    root_attr("status")
}

pub(crate) fn turbo_pct() -> PathBuf {
    root_attr("turbo_pct")
}
