pub(crate) mod path {
    use std::path::PathBuf;

    pub(crate) fn root() -> PathBuf {
        PathBuf::from("/sys/class/drm")
    }

    pub(crate) fn device(id: u64) -> PathBuf {
        let mut p = root();
        p.push(format!("card{}", id));
        p
    }

    pub(crate) fn device_attr(id: u64, a: &str) -> PathBuf {
        let mut p = device(id);
        p.push(a);
        p
    }

    pub(crate) fn bus_device(id: u64) -> PathBuf {
        device_attr(id, "device")
    }

    pub(crate) fn bus_device_attr(id: u64, a: &str) -> PathBuf {
        let mut p = bus_device(id);
        p.push(a);
        p
    }

    pub(crate) fn bus(id: u64) -> PathBuf {
        bus_device_attr(id, "subsystem")
    }

    pub(crate) fn driver(id: u64) -> PathBuf {
        bus_device_attr(id, "driver")
    }
}

use crate::{sysfs, Cached, Result};

pub async fn available() -> bool {
    path::root().is_dir()
}

pub async fn devices() -> Result<Vec<u64>> {
    sysfs::read_ids(&path::root(), "card").await
}

pub async fn bus(id: u64) -> Result<String> {
    sysfs::read_link_name(&path::bus(id)).await
}

pub async fn bus_id(id: u64) -> Result<String> {
    sysfs::read_link_name(&path::bus_device(id)).await
}

pub async fn driver(id: u64) -> Result<String> {
    sysfs::read_link_name(&path::driver(id)).await
}

#[derive(Clone, Debug)]
pub struct Card {
    id: u64,
    bus: Cached<String>,
    bus_id: Cached<String>,
    driver: Cached<String>,
}

impl Card {
    pub async fn available() -> bool {
        available().await
    }

    pub async fn ids() -> Result<Vec<u64>> {
        devices().await
    }

    pub fn new(id: u64) -> Self {
        let bus = Cached::default();
        let bus_id = Cached::default();
        let driver = Cached::default();
        Self {
            id,
            bus,
            bus_id,
            driver,
        }
    }

    pub async fn clear(&self) {
        tokio::join!(self.bus.clear(), self.bus_id.clear(), self.driver.clear(),);
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub async fn bus(&self) -> Result<String> {
        self.bus.get_with(bus(self.id)).await
    }

    pub async fn bus_id(&self) -> Result<String> {
        self.bus_id.get_with(bus_id(self.id)).await
    }

    pub async fn driver(&self) -> Result<String> {
        self.driver.get_with(driver(self.id)).await
    }
}
