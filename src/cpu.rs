pub(crate) mod path {
    use std::path::PathBuf;

    pub(crate) fn root() -> PathBuf {
        PathBuf::from("/sys/devices/system/cpu")
    }

    pub(crate) fn root_attr(a: &str) -> PathBuf {
        let mut p = root();
        p.push(a);
        p
    }

    pub(crate) fn device(id: u64) -> PathBuf {
        let mut p = root();
        p.push(format!("cpu{}", id));
        p
    }

    pub(crate) fn device_attr(i: u64, a: &str) -> PathBuf {
        let mut p = device(i);
        p.push(a);
        p
    }

    pub(crate) fn online_devices() -> PathBuf {
        root_attr("online")
    }

    pub(crate) fn offline_devices() -> PathBuf {
        root_attr("offline")
    }

    pub(crate) fn present_devices() -> PathBuf {
        root_attr("present")
    }

    pub(crate) fn possible_devices() -> PathBuf {
        root_attr("possible")
    }

    pub(crate) fn online(id: u64) -> PathBuf {
        device_attr(id, "online")
    }
}

use async_trait::async_trait;

use crate::sysfs::{self, Result};
use crate::{Feature, Multi, Read, Single, Values, Write, util};

pub async fn devices() -> Result<Vec<u64>> {
    sysfs::read_ids(&path::root(), "cpu").await
}

pub async fn online_devices() -> Result<Vec<u64>> {
    sysfs::read_indices(&path::online_devices()).await
}

pub async fn offline_devices() -> Result<Vec<u64>> {
    sysfs::read_indices(&path::offline_devices()).await
}

pub async fn present_devices() -> Result<Vec<u64>> {
    sysfs::read_indices(&path::present_devices()).await
}

pub async fn possible_devices() -> Result<Vec<u64>> {
    sysfs::read_indices(&path::possible_devices()).await
}

pub async fn online(id: u64) -> Result<bool> {
    sysfs::read_bool(&path::online(id)).await
}

pub async fn set_online(id: u64, val: bool) -> Result<()> {
    sysfs::write_bool(&path::online(id), val).await
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Device {
    pub id: u64,
    pub online: Option<bool>,
}

#[async_trait]
impl Read for Device {
    async fn read(&mut self) {
        self.online = online(self.id).await.ok();
    }
}

#[async_trait]
impl Write for Device {
    async fn write(&self) {
        if let Some(v) = self.online {
            let _ = set_online(self.id, v).await;
        }
    }
}

#[async_trait]
impl Values for Device {
    fn is_empty(&self) -> bool {
        self.eq(&Self::new(self.id))
    }

    fn clear(&mut self) {
        *self = Self::new(self.id);
    }
}

#[async_trait]
impl Multi for Device {
    type Id = u64;

    async fn ids() -> Vec<Self::Id> {
        devices().await.unwrap_or_default()
    }

    fn id(&self) -> Self::Id {
        self.id
    }

    fn set_id(&mut self, v: Self::Id) {
        self.id = v;
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
 pub struct System {
    pub devices: Vec<Device>,
}

#[async_trait]
impl Read for System {
    async fn read(&mut self) {
        self.devices.clear();
        self.devices.extend(Device::load_all().await);
    }
}

#[async_trait]
impl Write for System {
    async fn write(&self) {
        for device in &self.devices {
            device.write().await;
        }
        if self.devices.iter().any(|d| d.online.is_some()) {
            util::wait_for_cpu_onoff().await;
        }
    }
}

#[async_trait]
impl Values for System {
    fn is_empty(&self) -> bool {
        self.devices.is_empty()
    }

    fn clear(&mut self) {
        self.devices.clear();
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
