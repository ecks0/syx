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
use crate::util::sysfs::{self, Result};
use crate::{util, Feature, Multi, Read, Single, Values, Write};

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
pub struct Globals {
    max_perf_pct: Option<u64>,
    min_perf_pct: Option<u64>,
    no_turbo: Option<bool>,
    status: Option<String>,
    turbo_pct: Option<u64>,
}

impl Globals {
    pub fn max_perf_pct(&self) -> Option<u64> {
        self.max_perf_pct
    }

    pub fn min_perf_pct(&self) -> Option<u64> {
        self.min_perf_pct
    }

    pub fn no_turbo(&self) -> Option<bool> {
        self.no_turbo
    }

    pub fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }

    pub fn turbo_pct(&self) -> Option<u64> {
        self.turbo_pct
    }

    pub fn set_max_perf_pct(&mut self, v: impl Into<Option<u64>>) -> &mut Self {
        self.max_perf_pct = v.into();
        self
    }

    pub fn set_min_perf_pct(&mut self, v: impl Into<Option<u64>>) -> &mut Self {
        self.min_perf_pct = v.into();
        self
    }

    pub fn set_no_turbo(&mut self, v: impl Into<Option<bool>>) -> &mut Self {
        self.no_turbo = v.into();
        self
    }
}

#[async_trait]
impl Read for Globals {
    async fn read(&mut self) {
        self.max_perf_pct = max_perf_pct().await.ok();
        self.min_perf_pct = min_perf_pct().await.ok();
        self.no_turbo = no_turbo().await.ok();
        self.status = status().await.ok();
        self.turbo_pct = turbo_pct().await.ok();
    }
}

#[async_trait]
impl Write for Globals {
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
impl Values for Globals {
    fn is_empty(&self) -> bool {
        self.eq(&Self::default())
    }

    fn clear(&mut self) -> &mut Self {
        *self = Self::default();
        self
    }
}

#[async_trait]
impl Single for Globals {}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Device {
    id: u64,
    energy_perf_bias: Option<u64>,
    energy_performance_preference: Option<String>,
    energy_performance_available_preferences: Option<Vec<String>>,
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

    pub fn set_energy_perf_bias(&mut self, v: impl Into<Option<u64>>) -> &mut Self {
        self.energy_perf_bias = v.into();
        self
    }

    pub fn set_energy_performance_preference<O, S>(&mut self, v: O) -> &mut Self
    where
        O: Into<Option<S>>,
        S: Into<String>,
    {
        self.energy_performance_preference = v.into().map(|s| s.into());
        self
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

    fn clear(&mut self) -> &mut Self {
        *self = Self::new(self.id);
        self
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

    fn set_id(&mut self, v: Self::Id) -> &mut Self {
        self.id = v;
        self
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct System {
    globals: Globals,
    devices: Vec<Device>,
}

impl System {
    pub fn set_globals(&mut self, v: Globals) -> &mut Self {
        self.globals = v;
        self
    }

    pub fn globals(&self) -> &Globals {
        &self.globals
    }

    pub fn into_globals(self) -> Globals {
        self.globals
    }

    pub fn push_device(&mut self, v: Device) -> &mut Self {
        if let Some(i) = self.devices.iter().position(|d| v.id.eq(&d.id)) {
            self.devices[i] = v;
        } else {
            self.devices.push(v);
            self.devices.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        }
        self
    }

    pub fn push_devices(&mut self, v: impl IntoIterator<Item = Device>) -> &mut Self {
        for d in v.into_iter() {
            self.push_device(d);
        }
        self
    }

    pub fn devices(&self) -> std::slice::Iter<'_, Device> {
        self.devices.iter()
    }

    pub fn into_devices(self) -> impl IntoIterator<Item = Device> {
        self.devices.into_iter()
    }
}

#[async_trait]
impl Read for System {
    async fn read(&mut self) {
        self.globals.read().await;
        self.devices.clear();
        self.devices.extend(Device::load_all().await);
    }
}

#[async_trait]
impl Write for System {
    async fn write(&self) {
        self.globals.write().await;
        if !self.devices.is_empty() {
            let ids = self
                .devices
                .iter()
                .filter_map(|d| if d.is_empty() { None } else { Some(d.id) })
                .collect();
            let ids = util::cpu::set_online(ids).await;
            for device in &self.devices {
                device.write().await;
            }
            util::cpu::wait_for_write().await;
            util::cpu::set_offline(ids).await;
        }
    }
}

impl Values for System {
    fn is_empty(&self) -> bool {
        self.eq(&Self::default())
    }

    fn clear(&mut self) -> &mut Self {
        self.globals = Globals::default();
        self.devices.clear();
        self
    }
}

impl Single for System {}

#[async_trait]
impl Feature for System {
    async fn present() -> bool {
        path::status().is_file()
    }
}

impl From<Globals> for System {
    fn from(v: Globals) -> Self {
        let mut s = Self::default();
        s.set_globals(v);
        s
    }
}

impl From<Vec<Device>> for System {
    fn from(v: Vec<Device>) -> Self {
        let mut s = Self::default();
        for d in v {
            s.push_device(d);
        }
        s
    }
}
