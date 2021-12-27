use std::path::PathBuf;

use crate::BusId;

pub(crate) fn root() -> PathBuf {
    PathBuf::from("/sys/class/drm")
}

pub(crate) fn card(id: u64) -> PathBuf {
    let mut p = root();
    p.push(format!("card{}", id));
    p
}

pub(crate) fn card_attr(id: u64, a: &str) -> PathBuf {
    let mut p = card(id);
    p.push(a);
    p
}

pub(crate) fn device(id: u64) -> PathBuf {
    card_attr(id, "device")
}

pub(crate) fn device_attr(id: u64, a: &str) -> PathBuf {
    let mut p = device(id);
    p.push(a);
    p
}

pub(crate) fn subsystem(id: u64) -> PathBuf {
    device_attr(id, "subsystem")
}

pub(crate) fn driver(id: u64) -> PathBuf {
    device_attr(id, "driver")
}

pub(crate) fn bus_drm(bus_id: &BusId) -> PathBuf {
    let s = format!("/sys/bus/{}/devices/{}/drm", bus_id.bus, bus_id.id);
    PathBuf::from(s)
}
