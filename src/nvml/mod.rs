use std::result::Result as StdResult;
use std::sync::Arc;

use async_trait::async_trait;
use nvml_wrapper::enum_wrappers::device::{Clock as NVMLClock, ClockId as NVMLClockId};
pub use nvml_wrapper::error::NvmlError;
use nvml_wrapper::NVML;
use tokio::sync::{Mutex, OnceCell};
use tokio::task::spawn_blocking;

use crate::{Feature, Policy};

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
    let mut device = { with_nvml('r', "device_by_index", move |n| n.device_by_index(id)).await? };
    let res = spawn_blocking(move || f(&mut device)).await.unwrap();
    match res {
        Ok(v) => {
            log::debug!("OK nvml {} NVML::{}() {} {:?}", op, method, id, v);
            Ok(v)
        },
        Err(e) => {
            let msg = format!("ERR nvml {} Device::{}() {} {}", op, method, id, e);
            if op == W {
                log::error!("{}", msg);
            } else {
                log::warn!("{}", msg);
            }
            Err(e.into())
        },
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Device {
    pub id: u32,
    pub pci_id: Option<String>,
    pub gfx_freq_cur: Option<u32>,
    pub gfx_freq_min: Option<u32>,
    pub gfx_freq_max: Option<u32>,
    pub gfx_freq_reset: Option<bool>,
    pub mem_freq_cur: Option<u32>,
    pub mem_freq_max: Option<u32>,
    pub sm_freq_cur: Option<u32>,
    pub sm_freq_max: Option<u32>,
    pub video_freq_cur: Option<u32>,
    pub video_freq_max: Option<u32>,
    pub mem_total: Option<u64>,
    pub mem_used: Option<u64>,
    pub name: Option<String>,
    pub power_cur: Option<u32>,
    pub power_limit: Option<u32>,
    pub power_max: Option<u32>,
    pub power_min: Option<u32>,
}

#[async_trait]
impl Policy for Device {
    type Id = u32;
    type Output = Self;

    async fn ids() -> Vec<u32> {
        let c = with_nvml(R, "device_count", |n| n.device_count())
            .await
            .unwrap_or(0);
        (0u32..c).collect()
    }

    #[rustfmt::skip]
    async fn read(id: u32) -> Option<Self> {
        let mutex = mutex().await;
        let guard = mutex.lock().await;
        let pci_id = with_dev(id, R, "pci_info", |d| d.pci_info())
            .await
            .ok()
            .map(|i| i.bus_id);
        let gfx_freq_cur = with_dev(id, R, "clock", |d| {
            d.clock(NVMLClock::Graphics, NVMLClockId::Current)
        })
            .await
            .ok();
        let gfx_freq_min = None;
        let gfx_freq_max = with_dev(id, R, "max_clock_info", |d| {
            d.max_clock_info(NVMLClock::Graphics)
        })
            .await
            .ok();
        let gfx_freq_reset = None;
        let mem_freq_cur = with_dev(id, R, "clock", |d| {
            d.clock(NVMLClock::Memory, NVMLClockId::Current)
        })
            .await
            .ok();
        let mem_freq_max = with_dev(id, R, "max_clock_info", |d| {
            d.max_clock_info(NVMLClock::Memory)
        })
            .await
            .ok();
        let sm_freq_cur = with_dev(id, R, "clock", |d| {
            d.clock(NVMLClock::SM, NVMLClockId::Current)
        })
            .await
            .ok();
        let sm_freq_max = with_dev(id, R, "max_clock_info", |d| d.max_clock_info(NVMLClock::SM))
            .await
            .ok();
        let video_freq_cur = with_dev(id, R, "clock", |d| {
            d.clock(NVMLClock::Video, NVMLClockId::Current)
        })
            .await
            .ok();
        let video_freq_max = with_dev(id, R, "max_clock_info", |d| {
            d.max_clock_info(NVMLClock::Video)
        })
            .await
            .ok();
        let memory_info = with_dev(id, R, "memory_info", |d| d.memory_info()).await.ok();
        let mem_total = memory_info.as_ref().map(|i| i.total);
        let mem_used = memory_info.as_ref().map(|i| i.used);
        let name = with_dev(id, R, "name", |d| d.name()).await.ok();
        let power_cur = with_dev(id, R, "power_usage", |d| d.power_usage()).await.ok();
        let power_limit = with_dev(id, R, "enforced_power_limit", |d| d.enforced_power_limit())
            .await
            .ok();
        let power_constraints = with_dev(id, R, "power_management_limit_constraints", |d| {
            d.power_management_limit_constraints()
        })
            .await
            .ok();
        let power_max = power_constraints.as_ref().map(|c| c.max_limit);
        let power_min = power_constraints.as_ref().map(|c| c.min_limit);
        drop(guard);
        let s = Self {
            id,
            pci_id,
            gfx_freq_cur,
            gfx_freq_max,
            gfx_freq_min,
            gfx_freq_reset,
            mem_freq_cur,
            mem_freq_max,
            sm_freq_cur,
            sm_freq_max,
            video_freq_cur,
            video_freq_max,
            mem_total,
            mem_used,
            name,
            power_cur,
            power_limit,
            power_max,
            power_min,
        };
        Some(s)
    }

    async fn write(&self) {
        let mutex = mutex().await;
        let guard = mutex.lock().await;
        if let Some(val) = self.power_limit {
            let _ = with_dev(self.id, W, "set_power_management_limit", move |d| {
                d.set_power_management_limit(val)
            })
            .await;
        }
        if self.gfx_freq_reset.unwrap_or(false) {
            let _ = with_dev(self.id, W, "reset_gpu_locked_clocks", move |d| {
                d.reset_gpu_locked_clocks()
            })
            .await;
        }
        if let Some(min) = self.gfx_freq_min {
            if let Some(max) = self.gfx_freq_max {
                let _ = with_dev(self.id, W, "set_gpu_locked_clocks", move |d| {
                    d.set_gpu_locked_clocks(min, max)
                })
                .await;
            }
        }
        drop(guard);
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Nvml {
    pub devices: Vec<Device>,
}

#[async_trait]
impl Feature for Nvml {
    async fn present() -> bool {
        nvml().await.is_ok()
    }
}

#[async_trait]
impl Policy for Nvml {
    type Id = ();
    type Output = Self;

    async fn ids() -> Vec<()> {
        vec![()]
    }

    async fn read(_: ()) -> Option<Self> {
        let devices = Device::all().await;
        if devices.is_empty() {
            None
        } else {
            let s = Self { devices };
            Some(s)
        }
    }

    async fn write(&self) {
        for device in &self.devices {
            device.write().await;
        }
    }
}
