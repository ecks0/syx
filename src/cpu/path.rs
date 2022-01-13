use std::path::PathBuf;

pub(crate) fn root() -> PathBuf {
    PathBuf::from("/sys/devices/system/cpu")
}

pub(crate) fn root_attr(a: &str) -> PathBuf {
    root().join(a)
}

pub(crate) fn cpu(id: u64) -> PathBuf {
    root_attr(&format!("cpu{}", id))
}

pub(crate) fn cpu_attr(i: u64, a: &str) -> PathBuf {
    cpu(i).join(a)
}

pub(crate) fn online_ids() -> PathBuf {
    root_attr("online")
}

pub(crate) fn offline_ids() -> PathBuf {
    root_attr("offline")
}

pub(crate) fn present_ids() -> PathBuf {
    root_attr("present")
}

pub(crate) fn possible_ids() -> PathBuf {
    root_attr("possible")
}

pub(crate) fn online(id: u64) -> PathBuf {
    cpu_attr(id, "online")
}
