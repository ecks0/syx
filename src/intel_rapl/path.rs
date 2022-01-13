use std::path::PathBuf;

pub(crate) fn root() -> PathBuf {
    PathBuf::from("/sys/devices/virtual/powercap/intel-rapl")
}

pub(crate) fn package(package: u64) -> PathBuf {
    root().join(&format!("intel-rapl:{}", package))
}

pub(crate) fn subzone(package_: u64, subzone: u64) -> PathBuf {
    let mut p = package(package_);
    p.push(&format!("intel-rapl:{}:{}", package_, subzone));
    p
}

pub(crate) fn zone(package_: u64, subzone_: Option<u64>) -> PathBuf {
    match subzone_ {
        Some(subzone_) => subzone(package_, subzone_),
        None => package(package_),
    }
}

pub(crate) fn zone_attr(package: u64, subzone: Option<u64>, a: &str) -> PathBuf {
    zone(package, subzone).join(a)
}
