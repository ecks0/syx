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

use async_trait::async_trait;

use crate::util::sysfs;
use crate::{Feature, Multi, Read, Result, Single, Values};

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

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Device {
    id: u64,
    bus: Option<String>,
    bus_id: Option<String>,
    driver: Option<String>,
}

impl Device {
    pub fn bus(&self) -> Option<&str> {
        self.bus.as_deref()
    }

    pub fn bus_id(&self) -> Option<&str> {
        self.bus_id.as_deref()
    }

    pub fn driver(&self) -> Option<&str> {
        self.driver.as_deref()
    }
}

#[async_trait]
impl Read for Device {
    async fn read(&mut self) {
        self.bus = bus(self.id).await.ok();
        self.bus_id = bus_id(self.id).await.ok();
        self.driver = driver(self.id).await.ok();
    }
}

#[async_trait]
impl Values for Device {
    fn is_empty(&self) -> bool {
        self.eq(&Self::new(self.id))
    }

    fn clear(&mut self) -> &mut Self {
        *self = Self::default();
        self
    }
}

#[async_trait]
impl Multi for Device {
    type Id = u64;

    async fn ids() -> Vec<u64> {
        devices().await.unwrap_or_default()
    }

    fn id(&self) -> u64 {
        self.id
    }

    fn set_id(&mut self, v: u64) -> &mut Self {
        self.id = v;
        self
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct System {
    devices: Vec<Device>,
}

impl System {
    pub fn devices(&self) -> std::slice::Iter<'_, Device> {
        self.devices.iter()
    }

    pub fn into_devices(self) -> impl IntoIterator<Item = Device> {
        self.devices.into_iter()
    }
}

#[async_trait]
impl Read for System {
    async fn read(&mut self) {
        self.devices.clear();
        self.devices.extend(Device::load_all().await);
    }
}

#[async_trait]
impl Values for System {
    fn is_empty(&self) -> bool {
        self.devices.is_empty()
    }

    fn clear(&mut self) -> &mut Self {
        self.devices.clear();
        self
    }
}

#[async_trait]
impl Single for System {}

#[async_trait]
impl Feature for System {
    async fn present() -> bool {
        path::root().is_dir()
    }
}
