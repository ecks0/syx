pub(crate) mod path {
    use std::path::PathBuf;

    pub(crate) fn root() -> PathBuf {
        PathBuf::from("/sys/devices/system/cpu/cpufreq")
    }

    pub(crate) fn device(id: u64) -> PathBuf {
        let mut p = root();
        p.push(&format!("policy{}", id));
        p
    }

    pub(crate) fn device_attr(i: u64, a: &str) -> PathBuf {
        let mut p = device(i);
        p.push(a);
        p
    }

    pub(crate) fn cpuinfo_max_freq(id: u64) -> PathBuf {
        device_attr(id, "cpuinfo_max_freq")
    }

    pub(crate) fn cpuinfo_min_freq(id: u64) -> PathBuf {
        device_attr(id, "cpuinfo_min_freq")
    }

    pub(crate) fn scaling_cur_freq(id: u64) -> PathBuf {
        device_attr(id, "scaling_cur_freq")
    }

    pub(crate) fn scaling_driver(id: u64) -> PathBuf {
        device_attr(id, "scaling_driver")
    }

    pub(crate) fn scaling_governor(id: u64) -> PathBuf {
        device_attr(id, "scaling_governor")
    }

    pub(crate) fn scaling_available_governors(id: u64) -> PathBuf {
        device_attr(id, "scaling_available_governors")
    }

    pub(crate) fn scaling_max_freq(id: u64) -> PathBuf {
        device_attr(id, "scaling_max_freq")
    }

    pub(crate) fn scaling_min_freq(id: u64) -> PathBuf {
        device_attr(id, "scaling_min_freq")
    }
}

use async_trait::async_trait;

use crate::sysfs::{self, Result};
use crate::{Feature, Multi, Read, Single, Values, Write, util};

pub async fn devices() -> Result<Vec<u64>> {
    sysfs::read_ids(&path::root(), "policy").await
}

pub async fn cpuinfo_max_freq(id: u64) -> Result<u64> {
    sysfs::read_u64(&path::cpuinfo_max_freq(id)).await
}

pub async fn cpuinfo_min_freq(id: u64) -> Result<u64> {
    sysfs::read_u64(&path::cpuinfo_min_freq(id)).await
}

pub async fn scaling_cur_freq(id: u64) -> Result<u64> {
    sysfs::read_u64(&path::scaling_cur_freq(id)).await
}

pub async fn scaling_driver(id: u64) -> Result<String> {
    sysfs::read_str(&path::scaling_driver(id)).await
}

pub async fn scaling_governor(id: u64) -> Result<String> {
    sysfs::read_str(&path::scaling_governor(id)).await
}

pub async fn scaling_available_governors(id: u64) -> Result<Vec<String>> {
    sysfs::read_str_list(&path::scaling_available_governors(id), ' ').await
}

pub async fn scaling_max_freq(id: u64) -> Result<u64> {
    sysfs::read_u64(&path::scaling_max_freq(id)).await
}

pub async fn scaling_min_freq(id: u64) -> Result<u64> {
    sysfs::read_u64(&path::scaling_min_freq(id)).await
}

pub async fn set_scaling_governor(id: u64, val: &str) -> Result<()> {
    sysfs::write_str(&path::scaling_governor(id), val).await
}

pub async fn set_scaling_max_freq(id: u64, val: u64) -> Result<()> {
    sysfs::write_u64(&path::scaling_max_freq(id), val).await
}

pub async fn set_scaling_min_freq(id: u64, val: u64) -> Result<()> {
    sysfs::write_u64(&path::scaling_min_freq(id), val).await
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Device {
    pub id: u64,
    pub cpuinfo_max_freq: Option<u64>,
    pub cpuinfo_min_freq: Option<u64>,
    pub scaling_cur_freq: Option<u64>,
    pub scaling_driver: Option<String>,
    pub scaling_governor: Option<String>,
    pub scaling_available_governors: Option<Vec<String>>,
    pub scaling_max_freq: Option<u64>,
    pub scaling_min_freq: Option<u64>,
}

#[async_trait]
impl Read for Device {
    async fn read(&mut self) {
        self.cpuinfo_max_freq = cpuinfo_max_freq(self.id).await.ok();
        self.cpuinfo_min_freq = cpuinfo_min_freq(self.id).await.ok();
        self.scaling_cur_freq = scaling_cur_freq(self.id).await.ok();
        self.scaling_driver = scaling_driver(self.id).await.ok();
        self.scaling_governor = scaling_governor(self.id).await.ok();
        self.scaling_available_governors = scaling_available_governors(self.id).await.ok();
        self.scaling_max_freq = scaling_max_freq(self.id).await.ok();
        self.scaling_min_freq = scaling_min_freq(self.id).await.ok();
    }
}

#[async_trait]
impl Write for Device {
    async fn write(&self) {
        if let Some(v) = &self.scaling_governor {
            let _ = set_scaling_governor(self.id, v).await;
        }
        if let Some(v) = self.scaling_max_freq {
            let _ = set_scaling_max_freq(self.id, v).await;
        }
        if let Some(v) = self.scaling_min_freq {
            let _ = set_scaling_min_freq(self.id, v).await;
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
        if !self.devices.is_empty() {
            let ids = self
                .devices
                .iter()
                .filter_map(|d| if d.is_empty() { None } else { Some(d.id) })
                .collect();
            let ids = util::set_cpus_online(ids).await;
            for device in &self.devices {
                device.write().await;
            }
            util::wait_for_cpu_related().await;
            util::set_cpus_offline(ids).await;
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
