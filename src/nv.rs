use std::result::Result as StdResult;
use std::sync::Arc;

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

pub async fn gfx_freq(id: u32) -> Result<u32> {
    with_dev(id, R, "gfx_freq", |d| {
        d.clock(NVMLClock::Graphics, NVMLClockId::Current)
    })
    .await
}

pub async fn gfx_max_freq(id: u32) -> Result<u32> {
    with_dev(id, R, "gfx_max_freq", |d| {
        d.max_clock_info(NVMLClock::Graphics)
    })
    .await
}

pub async fn mem_freq(id: u32) -> Result<u32> {
    with_dev(id, R, "mem_freq", |d| {
        d.clock(NVMLClock::Memory, NVMLClockId::Current)
    })
    .await
}

pub async fn mem_max_freq(id: u32) -> Result<u32> {
    with_dev(id, R, "mem_max_freq", |d| {
        d.max_clock_info(NVMLClock::Memory)
    })
    .await
}

pub async fn sm_freq(id: u32) -> Result<u32> {
    with_dev(id, R, "sm_freq", |d| {
        d.clock(NVMLClock::SM, NVMLClockId::Current)
    })
    .await
}

pub async fn sm_max_freq(id: u32) -> Result<u32> {
    with_dev(id, R, "sm_max_freq", |d| d.max_clock_info(NVMLClock::SM)).await
}

pub async fn video_freq(id: u32) -> Result<u32> {
    with_dev(id, R, "video_freq", |d| {
        d.clock(NVMLClock::Video, NVMLClockId::Current)
    })
    .await
}

pub async fn video_max_freq(id: u32) -> Result<u32> {
    with_dev(id, R, "video_max_freq", |d| {
        d.max_clock_info(NVMLClock::Video)
    })
    .await
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

pub async fn power_usage(id: u32) -> Result<u32> {
    with_dev(id, R, "power_usage", |d| d.power_usage()).await
}

pub async fn power_limit(id: u32) -> Result<u32> {
    with_dev(id, R, "power_limit", |d| d.enforced_power_limit()).await
}

pub async fn power_max_limit(id: u32) -> Result<u32> {
    with_dev(id, R, "power_max_limit", |d| {
        d.power_management_limit_constraints()
    })
    .await
    .map(|c| c.max_limit)
}

pub async fn power_min_limit(id: u32) -> Result<u32> {
    with_dev(id, R, "power_min_limit", |d| {
        d.power_management_limit_constraints()
    })
    .await
    .map(|c| c.min_limit)
}

pub async fn set_gfx_freq(id: u32, min: u32, max: u32) -> Result<()> {
    with_dev(id, W, "set_gfx_freq", move |d| {
        d.set_gpu_locked_clocks(min, max)
    })
    .await
}

pub async fn gfx_freq_reset(id: u32) -> Result<()> {
    with_dev(id, W, "gfx_freq_reset", move |d| {
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

pub async fn power_limit_reset(id: u32) -> Result<()> {
    with_dev(id, W, "power_limit_reset", move |d| {
        d.set_power_management_limit(d.power_management_limit_default()?)
    })
    .await
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Device {
    id: u32,
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
    type Id = u32;

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
        let mutex = mutex().await;
        let guard = mutex.lock().await;
        let r = nvml().await.is_ok();
        drop(guard);
        r
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
