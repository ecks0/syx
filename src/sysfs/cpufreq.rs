pub mod path {
    use std::path::PathBuf;

    pub fn module() -> PathBuf {
        PathBuf::from("/sys/module/cpufreq")
    }

    pub fn root() -> PathBuf {
        PathBuf::from("/sys/devices/system/cpu/cpufreq")
    }

    pub fn device(id: u64) -> PathBuf {
        let mut p = root();
        p.push(&format!("policy{}", id));
        p
    }

    pub fn device_attr(id: u64, a: &str) -> PathBuf {
        let mut p = device(id);
        p.push(a);
        p
    }

    pub fn cpuinfo_max_freq(id: u64) -> PathBuf {
        device_attr(id, "cpuinfo_max_freq")
    }

    pub fn cpuinfo_min_freq(id: u64) -> PathBuf {
        device_attr(id, "cpuinfo_min_freq")
    }

    pub fn scaling_cur_freq(id: u64) -> PathBuf {
        device_attr(id, "scaling_cur_freq")
    }

    pub fn scaling_driver(id: u64) -> PathBuf {
        device_attr(id, "scaling_driver")
    }

    pub fn scaling_governor(id: u64) -> PathBuf {
        device_attr(id, "scaling_governor")
    }

    pub fn scaling_available_governors(id: u64) -> PathBuf {
        device_attr(id, "scaling_available_governors")
    }

    pub fn scaling_max_freq(id: u64) -> PathBuf {
        device_attr(id, "scaling_max_freq")
    }

    pub fn scaling_min_freq(id: u64) -> PathBuf {
        device_attr(id, "scaling_min_freq")
    }
}

use async_trait::async_trait;
use tokio::sync::OnceCell;

use crate::sysfs::{self, Result};
use crate::{Feature, Values};

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
impl Values for Device {
    type Id = u64;
    type Output = Self;

    async fn ids() -> Vec<u64> {
        devices().await.ok().unwrap_or_default()
    }

    async fn read(id: u64) -> Option<Self> {
        let cpuinfo_max_freq = cpuinfo_max_freq(id).await.ok();
        let cpuinfo_min_freq = cpuinfo_min_freq(id).await.ok();
        let scaling_cur_freq = scaling_cur_freq(id).await.ok();
        let scaling_driver = scaling_driver(id).await.ok();
        let scaling_governor = scaling_governor(id).await.ok();
        let scaling_available_governors = scaling_available_governors(id).await.ok();
        let scaling_max_freq = scaling_max_freq(id).await.ok();
        let scaling_min_freq = scaling_min_freq(id).await.ok();
        let s = Self {
            id,
            cpuinfo_max_freq,
            cpuinfo_min_freq,
            scaling_cur_freq,
            scaling_driver,
            scaling_governor,
            scaling_available_governors,
            scaling_max_freq,
            scaling_min_freq,
        };
        Some(s)
    }

    async fn write(&self) {
        if let Some(val) = &self.scaling_governor {
            let _ = set_scaling_governor(self.id, val).await;
        }
        if let Some(val) = self.scaling_max_freq {
            let _ = set_scaling_max_freq(self.id, val).await;
        }
        if let Some(val) = self.scaling_min_freq {
            let _ = set_scaling_min_freq(self.id, val).await;
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Cpufreq {
    pub devices: Vec<Device>,
}

#[async_trait]
impl Feature for Cpufreq {
    async fn present() -> bool {
        static PRESENT: OnceCell<bool> = OnceCell::const_new();
        async fn present() -> bool {
            path::module().is_dir()
        }
        *PRESENT.get_or_init(present).await
    }
}

#[async_trait]
impl Values for Cpufreq {
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
        let onlined = self.devices.iter().map(|d| d.id).collect();
        let onlined = crate::set_cpus_online(onlined).await;
        for device in &self.devices {
            device.write().await;
        }
        if !self.devices.is_empty() {
            crate::wait_for_cpu_related().await;
        }
        crate::set_cpus_offline(onlined).await;
    }
}
