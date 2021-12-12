use std::result::Result as StdResult;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use nvml_wrapper::enum_wrappers::device::{Clock as NVMLClock, ClockId as NVMLClockId};
use nvml_wrapper::error::NvmlError;
use nvml_wrapper::NVML;
use once_cell::sync::OnceCell;
use tokio::task::spawn_blocking;

use crate::{Error, Feature, Multi, Read, Result, Single, Values, Write};

fn nvml() -> Result<Arc<Mutex<NVML>>> {
    static INSTANCE: OnceCell<StdResult<Arc<Mutex<NVML>>, NvmlError>> = OnceCell::new();
    let r = INSTANCE.get_or_init(|| NVML::init().map(|nvml| Arc::new(Mutex::new(nvml))));
    match r {
        Ok(v) => Ok(Arc::clone(v)),
        Err(e) => Err(Error::nvml_init(e)),
    }
}

fn read_device_blocking<F, T>(id: u64, name: &'static str, f: F) -> Result<T>
where
    T: std::fmt::Debug,
    F: FnOnce(&nvml_wrapper::Device) -> StdResult<T, NvmlError>,
{
    let res = {
        let nvml = nvml()?;
        let nvml = nvml.lock().expect("nvml lock is poisoned");
        let device = nvml
            .device_by_index(id.try_into().unwrap())
            .map_err(|e| Error::nvml_read(id, name, e))?;
        f(&device).map_err(|e| Error::nvml_read(id, name, e))
    };
    #[cfg(feature = "logging")]
    match &res {
        Ok(v) => log::debug!("OK nvml r {} {} {:?}", name, id, v),
        Err(e) => log::warn!("ERR nvml w {} {} {}", name, id, e),
    }
    res
}

fn write_device_blocking<F, T>(id: u64, name: &'static str, f: F) -> Result<T>
where
    F: FnOnce(&mut nvml_wrapper::Device) -> StdResult<T, NvmlError>,
{
    let res = {
        let nvml = nvml()?;
        let nvml = nvml.lock().expect("nvml lock is poisoned");
        let mut device = nvml
            .device_by_index(id.try_into().unwrap())
            .map_err(|e| Error::nvml_write(id, name, e))?;
        f(&mut device).map_err(|e| Error::nvml_write(id, name, e))
    };
    #[cfg(feature = "logging")]
    match &res {
        Ok(_) => log::debug!("OK nvml w {} {}", name, id),
        Err(e) => log::error!("ERR nvml w {} {} {}", name, id, e),
    }
    res
}

async fn read_device<F, T>(id: u64, name: &'static str, f: F) -> Result<T>
where
    T: std::fmt::Debug + Send + 'static,
    F: FnOnce(&nvml_wrapper::Device) -> StdResult<T, NvmlError> + Send + 'static,
{
    spawn_blocking(move || read_device_blocking(id, name, f))
        .await
        .unwrap()
}

async fn write_device<F, T>(id: u64, name: &'static str, f: F) -> Result<T>
where
    T: std::fmt::Debug + Send + 'static,
    F: FnOnce(&mut nvml_wrapper::Device) -> StdResult<T, NvmlError> + Send + 'static,
{
    spawn_blocking(move || write_device_blocking(id, name, f))
        .await
        .unwrap()
}

fn devices_blocking() -> Result<Vec<u64>> {
    let nvml = nvml()?;
    let nvml = nvml.lock().expect("nvml lock is poisoned");
    let c = nvml.device_count().map_err(Error::NvmlListDevices)?;
    drop(nvml);
    let r = (0u64..c as u64).collect();
    Ok(r)
}

pub async fn devices() -> Result<Vec<u64>> {
    spawn_blocking(devices_blocking).await.unwrap()
}

pub async fn gfx_freq(id: u64) -> Result<u32> {
    read_device(id, "gfx_freq", |d| {
        d.clock(NVMLClock::Graphics, NVMLClockId::Current)
    })
    .await
}

pub async fn gfx_max_freq(id: u64) -> Result<u32> {
    read_device(id, "gfx_max_freq", |d| {
        d.max_clock_info(NVMLClock::Graphics)
    })
    .await
}

pub async fn mem_freq(id: u64) -> Result<u32> {
    read_device(id, "mem_freq", |d| {
        d.clock(NVMLClock::Memory, NVMLClockId::Current)
    })
    .await
}

pub async fn mem_max_freq(id: u64) -> Result<u32> {
    read_device(id, "mem_max_freq", |d| d.max_clock_info(NVMLClock::Memory)).await
}

pub async fn sm_freq(id: u64) -> Result<u32> {
    read_device(id, "sm_freq", |d| {
        d.clock(NVMLClock::SM, NVMLClockId::Current)
    })
    .await
}

pub async fn sm_max_freq(id: u64) -> Result<u32> {
    read_device(id, "sm_max_freq", |d| d.max_clock_info(NVMLClock::SM)).await
}

pub async fn video_freq(id: u64) -> Result<u32> {
    read_device(id, "video_freq", |d| {
        d.clock(NVMLClock::Video, NVMLClockId::Current)
    })
    .await
}

pub async fn video_max_freq(id: u64) -> Result<u32> {
    read_device(id, "video_max_freq", |d| d.max_clock_info(NVMLClock::Video)).await
}

pub async fn mem_total(id: u64) -> Result<u64> {
    read_device(id, "mem_total", |d| d.memory_info())
        .await
        .map(|i| i.total)
}

pub async fn mem_used(id: u64) -> Result<u64> {
    read_device(id, "mem_used", |d| d.memory_info())
        .await
        .map(|i| i.used)
}

pub async fn name(id: u64) -> Result<String> {
    read_device(id, "name", |d| d.name()).await
}

pub async fn power_usage(id: u64) -> Result<u32> {
    read_device(id, "power_usage", |d| d.power_usage()).await
}

pub async fn power_limit(id: u64) -> Result<u32> {
    read_device(id, "power_limit", |d| d.enforced_power_limit()).await
}

pub async fn power_max_limit(id: u64) -> Result<u32> {
    read_device(id, "power_max_limit", |d| {
        d.power_management_limit_constraints()
    })
    .await
    .map(|c| c.max_limit)
}

pub async fn power_min_limit(id: u64) -> Result<u32> {
    read_device(id, "power_min_limit", |d| {
        d.power_management_limit_constraints()
    })
    .await
    .map(|c| c.min_limit)
}

pub async fn set_gfx_freq(id: u64, min: u32, max: u32) -> Result<()> {
    write_device(id, "set_gfx_freq", move |d| {
        d.set_gpu_locked_clocks(min, max)
    })
    .await
}

pub async fn gfx_freq_reset(id: u64) -> Result<()> {
    write_device(id, "gfx_freq_reset", move |d| d.reset_gpu_locked_clocks()).await
}

pub async fn set_power_limit(id: u64, v: u32) -> Result<()> {
    write_device(id, "set_power_limit", move |d| {
        d.set_power_management_limit(v)
    })
    .await
}

pub async fn power_limit_reset(id: u64) -> Result<()> {
    write_device(id, "power_limit_reset", move |d| {
        d.set_power_management_limit(d.power_management_limit_default()?)
    })
    .await
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Device {
    id: u64,
    gfx_freq: Option<u32>,
    gfx_max_freq: Option<u32>,
    gfx_min_freq: Option<u32>,
    gfx_reset_freq: Option<bool>,
    mem_freq: Option<u32>,
    mem_max_freq: Option<u32>,
    sm_freq: Option<u32>,
    sm_max_freq: Option<u32>,
    video_freq: Option<u32>,
    video_max_freq: Option<u32>,
    mem_total: Option<u64>,
    mem_used: Option<u64>,
    name: Option<String>,
    power: Option<u32>,
    power_limit: Option<u32>,
    power_limit_max: Option<u32>,
    power_limit_min: Option<u32>,
    power_reset_limit: Option<bool>,
}

impl Device {
    pub fn gfx_freq(&self) -> Option<u32> {
        self.gfx_freq
    }

    pub fn gfx_max_freq(&self) -> Option<u32> {
        self.gfx_max_freq
    }

    pub fn gfx_min_freq(&self) -> Option<u32> {
        self.gfx_min_freq
    }

    pub fn gfx_freq_reset(&self) -> Option<bool> {
        self.gfx_reset_freq
    }

    pub fn mem_freq(&self) -> Option<u32> {
        self.mem_freq
    }

    pub fn mem_max_freq(&self) -> Option<u32> {
        self.mem_max_freq
    }

    pub fn sm_freq(&self) -> Option<u32> {
        self.sm_freq
    }

    pub fn video_freq(&self) -> Option<u32> {
        self.video_freq
    }

    pub fn video_max_freq(&self) -> Option<u32> {
        self.video_max_freq
    }

    pub fn mem_total(&self) -> Option<u64> {
        self.mem_total
    }

    pub fn mem_used(&self) -> Option<u64> {
        self.mem_used
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn power(&self) -> Option<u32> {
        self.power
    }

    pub fn power_limit(&self) -> Option<u32> {
        self.power_limit
    }

    pub fn power_limit_max(&self) -> Option<u32> {
        self.power_limit_max
    }

    pub fn power_limit_min(&self) -> Option<u32> {
        self.power_limit_min
    }

    pub fn power_limit_reset(&self) -> Option<bool> {
        self.power_reset_limit
    }

    pub fn set_gfx_max_freq(&mut self, v: impl Into<Option<u32>>) -> &mut Self {
        self.gfx_max_freq = v.into();
        self
    }

    pub fn set_gfx_min_freq(&mut self, v: impl Into<Option<u32>>) -> &mut Self {
        self.gfx_min_freq = v.into();
        self
    }

    pub fn set_gfx_reset_freq(&mut self, v: impl Into<Option<bool>>) -> &mut Self {
        self.gfx_reset_freq = v.into();
        self
    }

    pub fn set_power_limit(&mut self, v: impl Into<Option<u32>>) -> &mut Self {
        self.power_limit = v.into();
        self
    }

    pub fn set_power_reset_limit(&mut self, v: impl Into<Option<bool>>) -> &mut Self {
        self.power_reset_limit = v.into();
        self
    }
}

#[async_trait]
impl Read for Device {
    async fn read(&mut self) {
        self.gfx_freq = gfx_freq(self.id).await.ok();
        self.gfx_max_freq = gfx_max_freq(self.id).await.ok();
        self.gfx_min_freq = None;
        self.gfx_reset_freq = None;
        self.mem_freq = mem_freq(self.id).await.ok();
        self.mem_max_freq = mem_max_freq(self.id).await.ok();
        self.sm_freq = sm_freq(self.id).await.ok();
        self.sm_max_freq = sm_max_freq(self.id).await.ok();
        self.video_freq = video_freq(self.id).await.ok();
        self.video_max_freq = video_max_freq(self.id).await.ok();
        self.mem_total = mem_total(self.id).await.ok();
        self.mem_used = mem_used(self.id).await.ok();
        self.name = name(self.id).await.ok();
        self.power = power_usage(self.id).await.ok();
        self.power_limit = power_limit(self.id).await.ok();
        self.power_limit_max = power_max_limit(self.id).await.ok();
        self.power_limit_min = power_min_limit(self.id).await.ok();
        self.power_reset_limit = None;
    }
}

#[async_trait]
impl Write for Device {
    async fn write(&self) {
        if let Some(min) = self.gfx_min_freq {
            if let Some(max) = self.gfx_max_freq {
                let _ = set_gfx_freq(self.id, min, max).await;
            }
        }
        if self.gfx_reset_freq.unwrap_or(false) {
            let _ = gfx_freq_reset(self.id).await;
        }
        if let Some(v) = self.power_limit {
            let _ = set_power_limit(self.id, v).await;
        }
        if self.power_reset_limit.unwrap_or(false) {
            let _ = power_limit_reset(self.id).await;
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
    devices: Vec<Device>,
}

impl System {
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
        self.devices.clear();
        self.devices.extend(Device::load_all().await);
    }
}

#[async_trait]
impl Write for System {
    async fn write(&self) {
        for device in &self.devices {
            device.write().await;
        }
    }
}

#[async_trait]
impl Values for System {
    fn is_empty(&self) -> bool {
        self.devices.is_empty()
    }

    fn clear(&mut self) -> &mut Self {
        self.devices.clear();
        self
    }
}

#[async_trait]
impl Single for System {}

#[async_trait]
impl Feature for System {
    async fn present() -> bool {
        spawn_blocking(|| nvml().is_ok()).await.unwrap()
    }
}
