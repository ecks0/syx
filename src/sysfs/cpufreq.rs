pub mod path {
    use std::path::PathBuf;

    pub fn root() -> PathBuf {
        PathBuf::from("/sys/devices/system/cpu/cpufreq")
    }

    pub fn policy(id: u64) -> PathBuf {
        let mut p = root();
        p.push(&format!("policy{}", id));
        p
    }

    pub fn policy_attr(id: u64, a: &str) -> PathBuf {
        let mut p = policy(id);
        p.push(a);
        p
    }

    pub fn cpuinfo_max_freq(id: u64) -> PathBuf {
        policy_attr(id, "cpuinfo_max_freq")
    }

    pub fn cpuinfo_min_freq(id: u64) -> PathBuf {
        policy_attr(id, "cpuinfo_min_freq")
    }

    pub fn scaling_cur_freq(id: u64) -> PathBuf {
        policy_attr(id, "scaling_cur_freq")
    }

    pub fn scaling_driver(id: u64) -> PathBuf {
        policy_attr(id, "scaling_driver")
    }

    pub fn scaling_governor(id: u64) -> PathBuf {
        policy_attr(id, "scaling_governor")
    }

    pub fn scaling_available_governors(id: u64) -> PathBuf {
        policy_attr(id, "scaling_available_governors")
    }

    pub fn scaling_max_freq(id: u64) -> PathBuf {
        policy_attr(id, "scaling_max_freq")
    }

    pub fn scaling_min_freq(id: u64) -> PathBuf {
        policy_attr(id, "scaling_min_freq")
    }
}

use async_trait::async_trait;

use crate::sysfs::{self, Result};
use crate::{Feature, Resource};

pub async fn policies() -> Result<Vec<u64>> {
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

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Policy {
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
impl Resource for Policy {
    type Id = u64;
    type Output = Self;

    async fn ids() -> Vec<u64> {
        policies().await.ok().unwrap_or_default()
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

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Cpufreq {
    pub policies: Vec<Policy>,
}

#[async_trait]
impl Feature for Cpufreq {
    async fn present() -> bool {
        path::root().is_dir()
    }
}

#[async_trait]
impl Resource for Cpufreq {
    type Id = ();
    type Output = Self;

    async fn ids() -> Vec<Self::Id> {
        vec![()]
    }

    async fn read(_: ()) -> Option<Self> {
        let policies = Policy::all().await;
        let s = Self { policies };
        Some(s)
    }

    async fn write(&self) {
        for policy in &self.policies {
            policy.write().await;
        }
    }
}
