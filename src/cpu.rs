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

    pub(crate) fn online(id: u64) -> PathBuf {
        cpu_attr(id, "online")
    }
}

use crate::util::cell::Cell;
use crate::util::sysfs;
use crate::Result;

pub async fn available() -> Result<bool> {
    Ok(path::root().is_dir())
}

pub async fn exists(id: u64) -> Result<bool> {
    Ok(path::cpu(id).is_dir())
}

pub async fn ids() -> Result<Vec<u64>> {
    sysfs::read_ids(&path::root(), "cpu").await
}

pub async fn ids_online() -> Result<Vec<u64>> {
    sysfs::read_indices(&path::ids_online()).await
}

pub async fn ids_offline() -> Result<Vec<u64>> {
    sysfs::read_indices(&path::ids_offline()).await
}

pub async fn ids_present() -> Result<Vec<u64>> {
    sysfs::read_indices(&path::ids_present()).await
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
    online: Cell<bool>,
}

impl Cpu {
    pub async fn available() -> Result<bool> {
        available().await
    }

    pub async fn exists(id: u64) -> Result<bool> {
        exists(id).await
    }

    pub async fn ids() -> Result<Vec<u64>> {
        ids().await
    }

    pub fn new(id: u64) -> Self {
        let online = Cell::default();
        Self { id, online }
    }

    pub async fn clear(&self) {
        self.online.clear().await;
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub async fn online(&self) -> Result<bool> {
        self.online.get_or_load(online(self.id)).await
    }

    pub async fn set_online(&self, v: bool) -> Result<()> {
        self.online.clear_if_ok(set_online(self.id, v)).await
    }
}
