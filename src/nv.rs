use std::{result::Result as StdResult, sync::Arc};

use async_trait::async_trait;
use nvml_wrapper::enum_wrappers::device::{Clock as NVMLClock, ClockId as NVMLClockId};
pub use nvml_wrapper::error::NvmlError;
use nvml_wrapper::NVML;
use tokio::sync::{Mutex, OnceCell};
use tokio::task::spawn_blocking;

use crate::{Feature, Multi, Read, Single, Values, Write};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("nvml init failed: {0}")]
    Init(&'static NvmlError),

    #[error(transparent)]
    Nvml(#[from] NvmlError),
}

pub type Result<T> = StdResult<T, Error>;

async fn nvml() -> Result<&'static NVML> {
    async fn instance() -> StdResult<NVML, NvmlError> {
        spawn_blocking(NVML::init).await.unwrap()
    }
    static INSTANCE: OnceCell<StdResult<NVML, NvmlError>> = OnceCell::const_new();
    match INSTANCE.get_or_init(instance).await {
        Ok(v) => Ok(v),
        Err(e) => Err(Error::Init(e)),
    }
}

async fn mutex() -> Arc<Mutex<()>> {
    async fn mutex() -> Arc<Mutex<()>> {
        Arc::new(Mutex::new(()))
    }
    static MUTEX: OnceCell<Arc<Mutex<()>>> = OnceCell::const_new();
    MUTEX.get_or_init(mutex).await.clone()
}

const R: char = 'r';
const W: char = 'w';

async fn with_nvml<T, F>(op: char, method: &str, f: F) -> Result<T>
where
    T: std::fmt::Debug + Send + 'static,
    F: FnOnce(&'static NVML) -> StdResult<T, NvmlError> + Send + 'static,
{
    let nvml = nvml().await?;
    let res = spawn_blocking(|| f(nvml)).await.unwrap();
    match res {
        Ok(v) => {
            log::debug!("OK nvml {} NVML::{}() {:?}", op, method, v);
            Ok(v)
        },
        Err(e) => {
            let msg = format!("ERR nvml {} NVML::{}() {}", op, method, e);
            match e {
                NvmlError::DriverNotLoaded | NvmlError::LibraryNotFound => log::warn!("{}", msg),
                _ => log::error!("{}", msg),
            }
            Err(e.into())
        },
    }
}

async fn with_dev<T, F>(id: u32, op: char, method: &str, f: F) -> Result<T>
where
    T: std::fmt::Debug + Send + 'static,
    F: FnOnce(&mut nvml_wrapper::Device) -> StdResult<T, NvmlError> + Send + 'static,
{
    let mutex = mutex().await;
    let guard = mutex.lock().await;
    let mut device = { with_nvml('r', "device_by_index", move |n| n.device_by_index(id)).await? };
    let res = spawn_blocking(move || f(&mut device)).await.unwrap();
    drop(guard);
    match res {
        Ok(v) => {
            log::debug!("OK nvml {} {} {} {:?}", op, method, id, v);
            Ok(v)
        },
        Err(e) => {
            let msg = format!("ERR nvml {} {} {} {}", op, method, id, e);
            if op == W {
                log::error!("{}", msg);
            } else {
                log::warn!("{}", msg);
            }
            Err(e.into())
        },
    }
}

pub async fn devices() -> Result<Vec<u32>> {
    let mutex = mutex().await;
    let guard = mutex.lock().await;
    let c = with_nvml(R, "device_count", |n| n.device_count()).await?;
    drop(guard);
    let r = (0u32..c).collect();
    Ok(r)
}

pub async fn freq_gfx(id: u32) -> Result<u32> {
    with_dev(id, R, "freq_gfx", |d| d.clock(NVMLClock::Graphics, NVMLClockId::Current)).await
}

pub async fn freq_gfx_max(id: u32) -> Result<u32> {
    with_dev(id, R, "freq_gfx_max", |d| d.max_clock_info(NVMLClock::Graphics)).await
}

pub async fn freq_mem(id: u32) -> Result<u32> {
    with_dev(id, R, "freq_mem", |d| d.clock(NVMLClock::Memory, NVMLClockId::Current)).await
}

pub async fn freq_mem_max(id: u32) -> Result<u32> {
    with_dev(id, R, "freq_mem_max", |d| d.max_clock_info(NVMLClock::Memory)).await
}

pub async fn freq_sm(id: u32) -> Result<u32> {
    with_dev(id, R, "freq_sm", |d| d.clock(NVMLClock::SM, NVMLClockId::Current)).await
}

pub async fn freq_sm_max(id: u32) -> Result<u32> {
    with_dev(id, R, "freq_sm_max", |d| d.max_clock_info(NVMLClock::SM)).await
}

pub async fn freq_video(id: u32) -> Result<u32> {
    with_dev(id, R, "freq_video", |d| d.clock(NVMLClock::Video, NVMLClockId::Current)).await
}

pub async fn freq_video_max(id: u32) -> Result<u32> {
    with_dev(id, R, "freq_video_max", |d| d.max_clock_info(NVMLClock::Video)).await
}

pub async fn mem_total(id: u32) -> Result<u64> {
    with_dev(id, R, "mem_total", |d| d.memory_info())
        .await
        .map(|i| i.total)
}

pub async fn mem_used(id: u32) -> Result<u64> {
    with_dev(id, R, "mem_used", |d| d.memory_info())
        .await
        .map(|i| i.used)
}

pub async fn name(id: u32) -> Result<String> {
    with_dev(id, R, "name", |d| d.name()).await
}

pub async fn power(id: u32) -> Result<u32> {
    with_dev(id, R, "power", |d| d.power_usage()).await
}

pub async fn power_limit(id: u32) -> Result<u32> {
    with_dev(id, R, "power_limit", |d| d.enforced_power_limit()).await
}

pub async fn power_limit_max(id: u32) -> Result<u32> {
    with_dev(id, R, "power_limit_max", |d| d.power_management_limit_constraints())
        .await
        .map(|c| c.max_limit)
}

pub async fn power_limit_min(id: u32) -> Result<u32> {
    with_dev(id, R, "power_limit_min", |d| d.power_management_limit_constraints())
        .await
        .map(|c| c.min_limit)
}

pub async fn set_freq_gfx(id: u32, min: u32, max: u32) -> Result<()> {
    with_dev(id, W, "set_freq_gfx", move |d| {
        d.set_gpu_locked_clocks(min, max)
    })
    .await
}

pub async fn reset_freq_gfx(id: u32) -> Result<()> {
    with_dev(id, W, "reset_freq_gfx", move |d| {
        d.reset_gpu_locked_clocks()
    })
    .await
}

pub async fn set_power_limit(id: u32, v: u32) -> Result<()> {
    with_dev(id, W, "set_power_limit", move |d| {
        d.set_power_management_limit(v)
    })
       .await
}

pub async fn reset_power_limit(id: u32) -> Result<()> {
    with_dev(id, W, "reset_power_limit", move |d| {
        d.set_power_management_limit(d.power_management_limit_default()?)
    })
       .await
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Device {
    pub id: u32,
    pub freq_gfx: Option<u32>,
    pub freq_gfx_max: Option<u32>,
    pub freq_gfx_min: Option<u32>,
    pub freq_gfx_reset: Option<bool>,
    pub freq_mem: Option<u32>,
    pub freq_mem_max: Option<u32>,
    pub freq_sm: Option<u32>,
    pub freq_sm_max: Option<u32>,
    pub freq_video: Option<u32>,
    pub freq_video_max: Option<u32>,
    pub mem_total: Option<u64>,
    pub mem_used: Option<u64>,
    pub name: Option<String>,
    pub power: Option<u32>,
    pub power_limit: Option<u32>,
    pub power_limit_max: Option<u32>,
    pub power_limit_min: Option<u32>,
    pub power_limit_reset: Option<bool>,
}

#[async_trait]
impl Read for Device {
    async fn read(&mut self) {
        self.freq_gfx = freq_gfx(self.id).await.ok();
        self.freq_gfx_max = freq_gfx_max(self.id).await.ok();
        self.freq_gfx_min = None;
        self.freq_gfx_reset = None;
        self.freq_mem = freq_mem(self.id).await.ok();
        self.freq_mem_max = freq_mem_max(self.id).await.ok();
        self.freq_sm = freq_sm(self.id).await.ok();
        self.freq_sm_max = freq_sm_max(self.id).await.ok();
        self.freq_video = freq_video(self.id).await.ok();
        self.freq_video_max = freq_video_max(self.id).await.ok();
        self.mem_total = mem_total(self.id).await.ok();
        self.mem_used = mem_used(self.id).await.ok();
        self.name = name(self.id).await.ok();
        self.power = power(self.id).await.ok();
        self.power_limit = power_limit(self.id).await.ok();
        self.power_limit_max = power_limit_max(self.id).await.ok();
        self.power_limit_min = power_limit_min(self.id).await.ok();
        self.power_limit_reset = None;
    }
}

#[async_trait]
impl Write for Device {
    async fn write(&self) {
        if let Some(min) = self.freq_gfx_min {
            if let Some(max) = self.freq_gfx_max {
                let _ = set_freq_gfx(self.id, min, max).await;
            }
        }
        if self.freq_gfx_reset.unwrap_or(false) {
            let _ = reset_freq_gfx(self.id).await;
        }
        if let Some(v) = self.power_limit {
            let _ = set_power_limit(self.id, v).await;
        }
        if self.power_limit_reset.unwrap_or(false) {
            let _ = reset_power_limit(self.id).await;
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
    type Id = u32;

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

    fn clear(&mut self) {
        self.devices.clear();
    }
}

#[async_trait]
impl Single for System {}

#[async_trait]
impl Feature for System {
    async fn present() -> bool {
        let mutex = mutex().await;
        let guard = mutex.lock().await;
        let r = nvml().await.is_ok();
        drop(guard);
        r
    }
}
