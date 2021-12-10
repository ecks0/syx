pub(crate) mod path {
    use std::path::PathBuf;

    use crate::cpu::path::device_attr as cpu_device_attr;
    use crate::cpufreq::path::device_attr as cpufreq_device_attr;

    pub(crate) fn root() -> PathBuf {
        PathBuf::from("/sys/devices/system/cpu/intel_pstate")
    }

    pub(crate) fn root_attr(a: &str) -> PathBuf {
        let mut p = root();
        p.push(a);
        p
    }

    pub(crate) fn max_perf_pct() -> PathBuf {
        root_attr("max_perf_pct")
    }

    pub(crate) fn min_perf_pct() -> PathBuf {
        root_attr("min_perf_pct")
    }

    pub(crate) fn no_turbo() -> PathBuf {
        root_attr("no_turbo")
    }

    pub(crate) fn status() -> PathBuf {
        root_attr("status")
    }

    pub(crate) fn turbo_pct() -> PathBuf {
        root_attr("turbo_pct")
    }

    pub(crate) fn energy_perf_bias(id: u64) -> PathBuf {
        let mut p = cpu_device_attr(id, "power");
        p.push("energy_perf_bias");
        p
    }

    pub(crate) fn energy_performance_preference(id: u64) -> PathBuf {
        cpufreq_device_attr(id, "energy_performance_preference")
    }

    pub(crate) fn energy_performance_available_preferences(id: u64) -> PathBuf {
        cpufreq_device_attr(id, "energy_performance_available_preferences")
    }
}

use async_trait::async_trait;

pub use crate::cpufreq::devices;
use crate::sysfs::{self, Result};
use crate::{Feature, Multi, Read, Single, Values, Write, util};

pub async fn energy_perf_bias(id: u64) -> Result<u64> {
    sysfs::read_u64(&path::energy_perf_bias(id)).await
}

pub async fn energy_performance_preference(id: u64) -> Result<String> {
    sysfs::read_str(&path::energy_performance_preference(id)).await
}

pub async fn energy_performance_available_preferences(id: u64) -> Result<Vec<String>> {
    sysfs::read_str_list(&path::energy_performance_available_preferences(id), ' ').await
}

pub async fn max_perf_pct() -> Result<u64> {
    sysfs::read_u64(&path::max_perf_pct()).await
}

pub async fn min_perf_pct() -> Result<u64> {
    sysfs::read_u64(&path::min_perf_pct()).await
}

pub async fn no_turbo() -> Result<bool> {
    sysfs::read_bool(&path::no_turbo()).await
}

pub async fn status() -> Result<String> {
    sysfs::read_str(&path::status()).await
}

pub async fn turbo_pct() -> Result<u64> {
    sysfs::read_u64(&path::turbo_pct()).await
}

pub async fn set_energy_perf_bias(id: u64, v: u64) -> Result<()> {
    sysfs::write_u64(&path::energy_perf_bias(id), v).await
}

pub async fn set_energy_performance_preference(id: u64, v: &str) -> Result<()> {
    sysfs::write_str(&path::energy_performance_preference(id), v).await
}

pub async fn set_max_perf_pct(v: u64) -> Result<()> {
    sysfs::write_u64(&path::max_perf_pct(), v).await
}

pub async fn set_min_perf_pct(v: u64) -> Result<()> {
    sysfs::write_u64(&path::min_perf_pct(), v).await
}

pub async fn set_no_turbo(v: bool) -> Result<()> {
    sysfs::write_bool(&path::no_turbo(), v).await
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Global {
    pub max_perf_pct: Option<u64>,
    pub min_perf_pct: Option<u64>,
    pub no_turbo: Option<bool>,
    pub status: Option<String>,
    pub turbo_pct: Option<u64>,
}

#[async_trait]
impl Read for Global {
    async fn read(&mut self) {
        self.max_perf_pct = max_perf_pct().await.ok();
        self.min_perf_pct = min_perf_pct().await.ok();
        self.no_turbo = no_turbo().await.ok();
        self.status = status().await.ok();
        self.turbo_pct = turbo_pct().await.ok();
    }
}

#[async_trait]
impl Write for Global {
    async fn write(&self) {
        if let Some(val) = self.max_perf_pct {
            let _ = set_max_perf_pct(val);
        }
        if let Some(val) = self.min_perf_pct {
            let _ = set_min_perf_pct(val);
        }
        if let Some(val) = self.no_turbo {
            let _ = set_no_turbo(val);
        }
    }
}

#[async_trait]
impl Values for Global {
    fn is_empty(&self) -> bool {
        self.eq(&Self::default())
    }

    fn clear(&mut self) {
        *self = Self::default();
    }
}

#[async_trait]
impl Single for Global {}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Device {
    pub id: u64,
    pub energy_perf_bias: Option<u64>,
    pub energy_performance_preference: Option<String>,
    pub energy_performance_available_preferences: Option<Vec<String>>,
}

impl Device {
    pub fn energy_perf_bias(&self) -> Option<u64> {
        self.energy_perf_bias
    }

    pub fn energy_performance_preference(&self) -> Option<&str> {
        self.energy_performance_preference.as_deref()
    }

    pub fn energy_performance_available_preferences(&self) -> Option<&[String]> {
        self.energy_performance_available_preferences.as_deref()
    }

    pub fn set_energy_perf_bias(&mut self, v: u64) {
        self.energy_perf_bias = Some(v);
    }

    pub fn set_energy_performance_preference<S: Into<String>>(&mut self, v: S) {
        self.energy_performance_preference = Some(v.into());
    }
}

#[async_trait]
impl Read for Device {
    async fn read(&mut self) {
        self.energy_perf_bias = energy_perf_bias(self.id).await.ok();
        self.energy_performance_preference = energy_performance_preference(self.id).await.ok();
        self.energy_performance_available_preferences =
            energy_performance_available_preferences(self.id).await.ok();
    }
}

#[async_trait]
impl Write for Device {
    async fn write(&self) {
        if let Some(val) = self.energy_perf_bias {
            let _ = set_energy_perf_bias(self.id, val);
        }
        if let Some(val) = &self.energy_performance_preference {
            let _ = set_energy_performance_preference(self.id, val);
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
    pub global: Global,
    pub devices: Vec<Device>,
}

#[async_trait]
impl Read for System {
    async fn read(&mut self) {
        self.global.read().await;
        self.devices.clear();
        self.devices.extend(Device::load_all().await);
    }
}

#[async_trait]
impl Write for System {
    async fn write(&self) {
        self.global.write().await;
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

impl Values for System {
    fn is_empty(&self) -> bool {
        self.eq(&Self::default())
    }

    fn clear(&mut self) {
        self.global = Default::default();
        self.devices.clear();
    }
}

impl Single for System {}

#[async_trait]
impl Feature for System {
    async fn present() -> bool {
        path::status().is_file()
    }
}
