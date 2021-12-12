use std::result::Result as StdResult;
use std::sync::Arc;

use nvml_wrapper::enum_wrappers::device::{Clock as NVMLClock, ClockId as NVMLClockId};
use nvml_wrapper::error::NvmlError;
use nvml_wrapper::NVML;
use once_cell::sync::OnceCell;
use parking_lot::FairMutex;
use tokio::task::spawn_blocking;

use crate::{Cached, Error, Result};

fn nvml() -> Result<Arc<FairMutex<NVML>>> {
    static INSTANCE: OnceCell<StdResult<Arc<FairMutex<NVML>>, NvmlError>> = OnceCell::new();
    let r = INSTANCE.get_or_init(|| NVML::init().map(|nvml| Arc::new(FairMutex::new(nvml))));
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
        let nvml = nvml.lock();
        let device = nvml
            .device_by_index(id.try_into().unwrap())
            .map_err(|e| Error::nvml_read(id, name, e))?;
        let r = f(&device);
        drop(nvml);
        r.map_err(|e| Error::nvml_read(id, name, e))
    };
    #[cfg(feature = "logging")]
    match &res {
        Ok(v) => log::debug!("OK nvml r {} {} {:?}", name, id, v),
        Err(e) => log::warn!("ERR nvml r {} {} {}", name, id, e),
    }
    res
}

fn write_device_blocking<F, T>(id: u64, name: &'static str, f: F) -> Result<T>
where
    F: FnOnce(&mut nvml_wrapper::Device) -> StdResult<T, NvmlError>,
{
    let res = {
        let nvml = nvml()?;
        let nvml = nvml.lock();
        let mut device = nvml
            .device_by_index(id.try_into().unwrap())
            .map_err(|e| Error::nvml_write(id, name, e))?;
        let r = f(&mut device);
        drop(nvml);
        r.map_err(|e| Error::nvml_write(id, name, e))
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

pub async fn available() -> bool {
    spawn_blocking(|| nvml().is_ok()).await.unwrap()
}

fn devices_blocking() -> Result<Vec<u64>> {
    let nvml = nvml()?;
    let nvml = nvml.lock();
    let r = nvml.device_count();
    drop(nvml);
    let c = r.map_err(Error::NvmlListDevices)?;
    let v = (0u64..c as u64).collect();
    Ok(v)
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

pub async fn power(id: u64) -> Result<u32> {
    read_device(id, "power", |d| d.power_usage()).await
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

pub async fn reset_gfx_freq(id: u64) -> Result<()> {
    write_device(id, "reset_gfx_freq", move |d| d.reset_gpu_locked_clocks()).await
}

pub async fn set_power_limit(id: u64, v: u32) -> Result<()> {
    write_device(id, "set_power_limit", move |d| {
        d.set_power_management_limit(v)
    })
    .await
}

pub async fn reset_power_limit(id: u64) -> Result<()> {
    write_device(id, "reset_power_limit", move |d| {
        d.set_power_management_limit(d.power_management_limit_default()?)
    })
    .await
}

#[derive(Clone, Debug)]
pub struct Card {
    id: u64,
    gfx_freq: Cached<u32>,
    gfx_max_freq: Cached<u32>,
    mem_freq: Cached<u32>,
    mem_max_freq: Cached<u32>,
    sm_freq: Cached<u32>,
    sm_max_freq: Cached<u32>,
    video_freq: Cached<u32>,
    video_max_freq: Cached<u32>,
    mem_total: Cached<u64>,
    mem_used: Cached<u64>,
    name: Cached<String>,
    power: Cached<u32>,
    power_limit: Cached<u32>,
    power_limit_max: Cached<u32>,
    power_limit_min: Cached<u32>,
}

impl Card {
    pub async fn available() -> bool {
        available().await
    }

    pub async fn ids() -> Result<Vec<u64>> {
        devices().await
    }

    pub fn new(id: u64) -> Self {
        let gfx_freq = Cached::default();
        let gfx_max_freq = Cached::default();
        let mem_freq = Cached::default();
        let mem_max_freq = Cached::default();
        let sm_freq = Cached::default();
        let sm_max_freq = Cached::default();
        let video_freq = Cached::default();
        let video_max_freq = Cached::default();
        let mem_total = Cached::default();
        let mem_used = Cached::default();
        let name = Cached::default();
        let power = Cached::default();
        let power_limit = Cached::default();
        let power_limit_max = Cached::default();
        let power_limit_min = Cached::default();
        Self {
            id,
            gfx_freq,
            gfx_max_freq,
            mem_freq,
            mem_max_freq,
            sm_freq,
            sm_max_freq,
            video_freq,
            video_max_freq,
            mem_total,
            mem_used,
            name,
            power,
            power_limit,
            power_limit_max,
            power_limit_min,
        }
    }

    pub async fn clear(&self) {
        tokio::join!(
            self.gfx_freq.clear(),
            self.gfx_max_freq.clear(),
            self.mem_freq.clear(),
            self.mem_max_freq.clear(),
            self.sm_freq.clear(),
            self.sm_max_freq.clear(),
            self.video_freq.clear(),
            self.video_max_freq.clear(),
            self.mem_total.clear(),
            self.mem_used.clear(),
            self.name.clear(),
            self.power.clear(),
            self.power_limit.clear(),
            self.power_limit_max.clear(),
            self.power_limit_min.clear(),
        );
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub async fn gfx_freq(&self) -> Result<u32> {
        self.gfx_freq.get_with(gfx_freq(self.id)).await
    }

    pub async fn gfx_max_freq(&self) -> Result<u32> {
        self.gfx_max_freq.get_with(gfx_max_freq(self.id)).await
    }

    pub async fn mem_freq(&self) -> Result<u32> {
        self.mem_freq.get_with(mem_freq(self.id)).await
    }

    pub async fn mem_max_freq(&self) -> Result<u32> {
        self.mem_max_freq.get_with(mem_max_freq(self.id)).await
    }

    pub async fn sm_freq(&self) -> Result<u32> {
        self.sm_freq.get_with(sm_freq(self.id)).await
    }

    pub async fn video_freq(&self) -> Result<u32> {
        self.video_freq.get_with(video_freq(self.id)).await
    }

    pub async fn video_max_freq(&self) -> Result<u32> {
        self.video_max_freq.get_with(video_max_freq(self.id)).await
    }

    pub async fn mem_total(&self) -> Result<u64> {
        self.mem_total.get_with(mem_total(self.id)).await
    }

    pub async fn mem_used(&self) -> Result<u64> {
        self.mem_used.get_with(mem_used(self.id)).await
    }

    pub async fn name(&self) -> Result<String> {
        self.name.get_with(name(self.id)).await
    }

    pub async fn power(&self) -> Result<u32> {
        self.power.get_with(power(self.id)).await
    }

    pub async fn power_limit(&self) -> Result<u32> {
        self.power_limit.get_with(power_limit(self.id)).await
    }

    pub async fn power_max_limit(&self) -> Result<u32> {
        self.power_limit_max
            .get_with(power_max_limit(self.id))
            .await
    }

    pub async fn power_min_limit(&self) -> Result<u32> {
        self.power_limit_min
            .get_with(power_min_limit(self.id))
            .await
    }

    pub async fn set_gfx_freq(&self, min: u32, max: u32) -> Result<()> {
        let f = set_gfx_freq(self.id, min, max);
        self.gfx_freq.clear_if(f).await
    }

    pub async fn reset_gfx_freq(&self) -> Result<()> {
        let f = reset_gfx_freq(self.id);
        self.gfx_freq.clear_if(f).await
    }

    pub async fn set_power_limit(&self, v: u32) -> Result<()> {
        let f = set_power_limit(self.id, v);
        self.power_limit.clear_if(f).await
    }

    pub async fn reset_power_limit(&self) -> Result<()> {
        let f = reset_power_limit(self.id);
        self.power_limit.clear_if(f).await
    }
}
