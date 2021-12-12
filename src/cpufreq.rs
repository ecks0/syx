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

use crate::{sysfs, Cached, Result};

pub async fn available() -> bool {
    path::root().is_dir()
}

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
pub struct Cpu {
    id: u64,
    cpuinfo_max_freq: Cached<u64>,
    cpuinfo_min_freq: Cached<u64>,
    scaling_cur_freq: Cached<u64>,
    scaling_driver: Cached<String>,
    scaling_governor: Cached<String>,
    scaling_available_governors: Cached<Vec<String>>,
    scaling_max_freq: Cached<u64>,
    scaling_min_freq: Cached<u64>,
}

impl Cpu {
    pub async fn available() -> bool {
        available().await
    }

    pub async fn ids() -> Result<Vec<u64>> {
        devices().await
    }

    pub fn new(id: u64) -> Self {
        let cpuinfo_max_freq = Cached::default();
        let cpuinfo_min_freq = Cached::default();
        let scaling_cur_freq = Cached::default();
        let scaling_driver = Cached::default();
        let scaling_governor = Cached::default();
        let scaling_available_governors = Cached::default();
        let scaling_max_freq = Cached::default();
        let scaling_min_freq = Cached::default();
        Self {
            id,
            cpuinfo_max_freq,
            cpuinfo_min_freq,
            scaling_cur_freq,
            scaling_driver,
            scaling_governor,
            scaling_available_governors,
            scaling_max_freq,
            scaling_min_freq,
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
            .get_with(cpuinfo_max_freq(self.id))
            .await
    }

    pub async fn cpuinfo_min_freq(&self) -> Result<u64> {
        self.cpuinfo_min_freq
            .get_with(cpuinfo_min_freq(self.id))
            .await
    }

    pub async fn scaling_cur_freq(&self) -> Result<u64> {
        self.scaling_cur_freq
            .get_with(scaling_cur_freq(self.id))
            .await
    }

    pub async fn scaling_driver(&self) -> Result<String> {
        self.scaling_driver.get_with(scaling_driver(self.id)).await
    }

    pub async fn scaling_governor(&self) -> Result<String> {
        self.scaling_governor
            .get_with(scaling_governor(self.id))
            .await
    }

    pub async fn scaling_available_governors(&self) -> Result<Vec<String>> {
        self.scaling_available_governors
            .get_with(scaling_available_governors(self.id))
            .await
    }

    pub async fn scaling_max_freq(&self) -> Result<u64> {
        self.scaling_max_freq
            .get_with(scaling_max_freq(self.id))
            .await
    }

    pub async fn scaling_min_freq(&self) -> Result<u64> {
        self.scaling_min_freq
            .get_with(scaling_min_freq(self.id))
            .await
    }

    pub async fn set_scaling_governor(&self, v: impl AsRef<str>) -> Result<()> {
        let f = set_scaling_governor(self.id, v.as_ref());
        self.scaling_governor.clear_if(f).await
    }

    pub async fn set_scaling_max_freq(&self, v: u64) -> Result<()> {
        let f = set_scaling_max_freq(self.id, v);
        self.scaling_max_freq.clear_if(f).await
    }

    pub async fn set_scaling_min_freq(&self, v: u64) -> Result<()> {
        let f = set_scaling_min_freq(self.id, v);
        self.scaling_min_freq.clear_if(f).await
    }
}
