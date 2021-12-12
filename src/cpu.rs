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

use crate::util::{self, sysfs};
use crate::{Feature, Multi, Read, Result, Single, Values, Write};

pub async fn devices() -> Result<Vec<u64>> {
    sysfs::read_ids(&path::root(), "cpu").await
}

pub async fn devices_online() -> Result<Vec<u64>> {
    sysfs::read_indices(&path::online_devices()).await
}

pub async fn devices_offline() -> Result<Vec<u64>> {
    sysfs::read_indices(&path::offline_devices()).await
}

pub async fn devices_present() -> Result<Vec<u64>> {
    sysfs::read_indices(&path::present_devices()).await
}

pub async fn devices_possible() -> Result<Vec<u64>> {
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
    id: u64,
    online: Option<bool>,
}

impl Device {
    pub fn online(&self) -> Option<bool> {
        self.online
    }

    pub fn set_online(&mut self, v: impl Into<Option<bool>>) -> &mut Self {
        self.online = v.into();
        self
    }
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
        self.online.is_none()
    }

    fn clear(&mut self) -> &mut Self {
        self.online = None;
        self
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

    fn set_id(&mut self, v: Self::Id) -> &mut Self {
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
    pub fn push_device(&mut self, v: Device) -> &mut Self {
        if let Some(i) = self.devices.iter().position(|d| v.id.eq(&d.id)) {
            self.devices[i] = v;
        } else {
            self.devices.push(v);
            self.devices.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        }
        self
    }

    pub fn push_devices(&mut self, v: impl IntoIterator<Item = Device>) -> &mut Self {
        for d in v.into_iter() {
            self.push_device(d);
        }
        self
    }

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
impl Write for System {
    async fn write(&self) {
        for device in &self.devices {
            device.write().await;
        }
        if self.devices.iter().any(|d| d.online.is_some()) {
            util::cpu::wait_for_onoff().await;
        }
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
