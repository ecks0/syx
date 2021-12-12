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

use crate::{sysfs, Cached, Result};

pub async fn available() -> bool {
    path::root().is_dir()
}

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

#[derive(Clone, Debug)]
pub struct Cpu {
    id: u64,
    online: Cached<bool>,
}

impl Cpu {
    pub async fn available() -> bool {
        available().await
    }

    pub async fn ids() -> Result<Vec<u64>> {
        devices().await
    }

    pub fn new(id: u64) -> Self {
        let online = Cached::default();
        Self { id, online }
    }

    pub async fn clear(&self) {
        self.online.clear().await;
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub async fn online(&self) -> Result<bool> {
        self.online.get_with(online(self.id)).await
    }

    pub async fn set_online(&self, v: bool) -> Result<()> {
        self.online.clear_if(set_online(self.id, v)).await
    }
}
