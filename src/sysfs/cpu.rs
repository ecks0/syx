pub mod path {
    use std::path::PathBuf;

    pub fn root() -> PathBuf {
        PathBuf::from("/sys/devices/system/cpu")
    }

    pub fn device(id: u64) -> PathBuf {
        let mut p = root();
        p.push(format!("cpu{}", id));
        p
    }

    pub fn device_attr(id: u64, a: &str) -> PathBuf {
        let mut p = device(id);
        p.push(a);
        p
    }

    pub fn online(id: u64) -> PathBuf {
        device_attr(id, "online")
    }
}

use async_trait::async_trait;

use crate::sysfs::{self, Result};
use crate::{Feature, Policy};

pub async fn devices() -> Result<Vec<u64>> {
    sysfs::read_ids(&path::root(), "cpu").await
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
impl Policy for Device {
    type Id = u64;
    type Output = Self;

    async fn ids() -> Vec<u64> {
        devices().await.ok().unwrap_or_default()
    }

    async fn read(id: u64) -> Option<Self> {
        let online = online(id).await.ok();
        let s = Self { id, online };
        Some(s)
    }

    async fn write(&self) {
        if let Some(v) = self.online {
            let _ = set_online(self.id, v);
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Cpu {
    pub devices: Vec<Device>,
}

#[async_trait]
impl Feature for Cpu {
    async fn present() -> bool {
        path::root().is_dir()
    }
}

#[async_trait]
impl Policy for Cpu {
    type Id = ();
    type Output = Self;

    async fn ids() -> Vec<()> {
        vec![()]
    }

    async fn read(_: ()) -> Option<Self> {
        let devices = Device::all().await;
        let s = Self { devices };
        Some(s)
    }

    async fn write(&self) {
        for device in &self.devices {
            device.write().await;
        }
    }
}
