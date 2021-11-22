pub mod path {
    use std::path::PathBuf;

    pub fn root() -> PathBuf {
        PathBuf::from("/sys/devices/system/cpu")
    }

    pub fn root_attr(a: &str) -> PathBuf {
        let mut p = root();
        p.push(a);
        p
    }

    pub fn devices_online() -> PathBuf {
        root_attr("online")
    }

    pub fn devices_offline() -> PathBuf {
        root_attr("offline")
    }

    pub fn devices_present() -> PathBuf {
        root_attr("present")
    }

    pub fn devices_possible() -> PathBuf {
        root_attr("possible")
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
use tokio::sync::OnceCell;

use crate::sysfs::{self, Result};
use crate::{Feature, Values};

pub async fn devices() -> Result<Vec<u64>> {
    sysfs::read_ids(&path::root(), "cpu").await
}

pub async fn devices_online() -> Result<Vec<u64>> {
    sysfs::read_indices(&path::devices_online()).await
}

pub async fn devices_offline() -> Result<Vec<u64>> {
    sysfs::read_indices(&path::devices_offline()).await
}

pub async fn devices_present() -> Result<Vec<u64>> {
    sysfs::read_indices(&path::devices_present()).await
}

pub async fn devices_possible() -> Result<Vec<u64>> {
    sysfs::read_indices(&path::devices_possible()).await
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
impl Values for Device {
    type Id = u64;
    type Output = Self;

    async fn ids() -> Vec<u64> {
        devices().await.unwrap_or_default()
    }

    async fn read(id: u64) -> Option<Self> {
        let online = online(id).await.ok();
        let s = Self { id, online };
        Some(s)
    }

    async fn write(&self) {
        if let Some(v) = self.online {
            let _ = set_online(self.id, v).await;
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
        static PRESENT: OnceCell<bool> = OnceCell::const_new();
        async fn present() -> bool {
            path::root().is_dir()
        }
        *PRESENT.get_or_init(present).await
    }
}

#[async_trait]
impl Values for Cpu {
    type Id = ();
    type Output = Self;

    async fn ids() -> Vec<()> {
        vec![()]
    }

    async fn read(_: ()) -> Option<Self> {
        if !Self::present().await {
            return None;
        }
        let devices = Device::all().await;
        let s = Self { devices };
        Some(s)
    }

    async fn write(&self) {
        for device in &self.devices {
            device.write().await;
        }
        if self.devices.iter().any(|d| d.online.is_some()) {
            crate::wait_for_cpu_onoff().await;
        }
    }
}
