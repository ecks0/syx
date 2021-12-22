pub(crate) mod path {
    use std::path::PathBuf;

    pub(crate) fn root() -> PathBuf {
        PathBuf::from("/sys/devices/system/cpu/cpufreq")
    }

    pub(crate) fn policy(id: u64) -> PathBuf {
        let mut p = root();
        p.push(&format!("policy{}", id));
        p
    }

    pub(crate) fn policy_attr(i: u64, a: &str) -> PathBuf {
        let mut p = policy(i);
        p.push(a);
        p
    }

    pub(crate) fn cpuinfo_max_freq(id: u64) -> PathBuf {
        policy_attr(id, "cpuinfo_max_freq")
    }

    pub(crate) fn cpuinfo_min_freq(id: u64) -> PathBuf {
        policy_attr(id, "cpuinfo_min_freq")
    }

    pub(crate) fn scaling_cur_freq(id: u64) -> PathBuf {
        policy_attr(id, "scaling_cur_freq")
    }

    pub(crate) fn scaling_driver(id: u64) -> PathBuf {
        policy_attr(id, "scaling_driver")
    }

    pub(crate) fn scaling_governor(id: u64) -> PathBuf {
        policy_attr(id, "scaling_governor")
    }

    pub(crate) fn scaling_available_governors(id: u64) -> PathBuf {
        policy_attr(id, "scaling_available_governors")
    }

    pub(crate) fn scaling_max_freq(id: u64) -> PathBuf {
        policy_attr(id, "scaling_max_freq")
    }

    pub(crate) fn scaling_min_freq(id: u64) -> PathBuf {
        policy_attr(id, "scaling_min_freq")
    }
}

use crate::util::cell::Cell;
use crate::util::sysfs;
use crate::Result;

pub async fn available() -> Result<bool> {
    Ok(path::root().is_dir())
}

pub async fn exists(id: u64) -> Result<bool> {
    Ok(path::policy(id).is_dir())
}

pub async fn ids() -> Result<Vec<u64>> {
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
    sysfs::read_string(&path::scaling_driver(id)).await
}

pub async fn scaling_governor(id: u64) -> Result<String> {
    sysfs::read_string(&path::scaling_governor(id)).await
}

pub async fn scaling_available_governors(id: u64) -> Result<Vec<String>> {
    sysfs::read_string_list(&path::scaling_available_governors(id), ' ').await
}

pub async fn scaling_max_freq(id: u64) -> Result<u64> {
    sysfs::read_u64(&path::scaling_max_freq(id)).await
}

pub async fn scaling_min_freq(id: u64) -> Result<u64> {
    sysfs::read_u64(&path::scaling_min_freq(id)).await
}

pub async fn set_scaling_governor(id: u64, val: &str) -> Result<()> {
    sysfs::write_string(&path::scaling_governor(id), val).await
}

pub async fn set_scaling_max_freq(id: u64, val: u64) -> Result<()> {
    sysfs::write_u64(&path::scaling_max_freq(id), val).await
}

pub async fn set_scaling_min_freq(id: u64, val: u64) -> Result<()> {
    sysfs::write_u64(&path::scaling_min_freq(id), val).await
}

#[derive(Clone, Debug)]
pub struct Policy {
    id: u64,
    cpuinfo_max_freq: Cell<u64>,
    cpuinfo_min_freq: Cell<u64>,
    scaling_cur_freq: Cell<u64>,
    scaling_driver: Cell<String>,
    scaling_governor: Cell<String>,
    scaling_available_governors: Cell<Vec<String>>,
    scaling_max_freq: Cell<u64>,
    scaling_min_freq: Cell<u64>,
}

impl Policy {
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
        Self {
            id,
            cpuinfo_max_freq: Cell::default(),
            cpuinfo_min_freq: Cell::default(),
            scaling_cur_freq: Cell::default(),
            scaling_driver: Cell::default(),
            scaling_governor: Cell::default(),
            scaling_available_governors: Cell::default(),
            scaling_max_freq: Cell::default(),
            scaling_min_freq: Cell::default(),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub async fn clear(&self) {
        tokio::join!(
            self.cpuinfo_max_freq.clear(),
            self.cpuinfo_min_freq.clear(),
            self.scaling_cur_freq.clear(),
            self.scaling_driver.clear(),
            self.scaling_governor.clear(),
            self.scaling_available_governors.clear(),
            self.scaling_max_freq.clear(),
            self.scaling_min_freq.clear(),
        );
    }

    pub async fn cpuinfo_max_freq(&self) -> Result<u64> {
        self.cpuinfo_max_freq
            .get_or_load(cpuinfo_max_freq(self.id))
            .await
    }

    pub async fn cpuinfo_min_freq(&self) -> Result<u64> {
        self.cpuinfo_min_freq
            .get_or_load(cpuinfo_min_freq(self.id))
            .await
    }

    pub async fn scaling_cur_freq(&self) -> Result<u64> {
        self.scaling_cur_freq
            .get_or_load(scaling_cur_freq(self.id))
            .await
    }

    pub async fn scaling_driver(&self) -> Result<String> {
        self.scaling_driver
            .get_or_load(scaling_driver(self.id))
            .await
    }

    pub async fn scaling_governor(&self) -> Result<String> {
        self.scaling_governor
            .get_or_load(scaling_governor(self.id))
            .await
    }

    pub async fn scaling_available_governors(&self) -> Result<Vec<String>> {
        self.scaling_available_governors
            .get_or_load(scaling_available_governors(self.id))
            .await
    }

    pub async fn scaling_max_freq(&self) -> Result<u64> {
        self.scaling_max_freq
            .get_or_load(scaling_max_freq(self.id))
            .await
    }

    pub async fn scaling_min_freq(&self) -> Result<u64> {
        self.scaling_min_freq
            .get_or_load(scaling_min_freq(self.id))
            .await
    }

    pub async fn set_scaling_governor(&self, v: impl AsRef<str>) -> Result<()> {
        self.scaling_governor
            .clear_if_ok(set_scaling_governor(self.id, v.as_ref()))
            .await
    }

    pub async fn set_scaling_max_freq(&self, v: u64) -> Result<()> {
        self.scaling_max_freq
            .clear_if_ok(set_scaling_max_freq(self.id, v))
            .await
    }

    pub async fn set_scaling_min_freq(&self, v: u64) -> Result<()> {
        self.scaling_min_freq
            .clear_if_ok(set_scaling_min_freq(self.id, v))
            .await
    }
}