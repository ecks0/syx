use std::path::PathBuf;

pub(crate) fn root() -> PathBuf {
    PathBuf::from("/sys/devices/system/cpu")
}

pub(crate) fn root_attr(a: &str) -> PathBuf {
    let mut p = root();
    p.push(a);
    p
}

pub(crate) fn cpu(id: u64) -> PathBuf {
    let mut p = root();
    p.push(format!("cpu{}", id));
    p
}

pub(crate) fn cpu_attr(i: u64, a: &str) -> PathBuf {
    let mut p = cpu(i);
    p.push(a);
    p
}

pub(crate) fn ids_online() -> PathBuf {
    root_attr("online")
}

pub(crate) fn ids_offline() -> PathBuf {
    root_attr("offline")
}

pub(crate) fn ids_present() -> PathBuf {
    root_attr("present")
}

pub(crate) fn ids_possible() -> PathBuf {
    root_attr("possible")
}

pub(crate) fn online(id: u64) -> PathBuf {
    cpu_attr(id, "online")
}
