use std::result::Result as StdResult;
use std::sync::Arc;

use nvml_wrapper::enum_wrappers::device::{Clock as NVMLClock, ClockId as NVMLClockId};
use nvml_wrapper::error::NvmlError;
use nvml_wrapper::NVML;
use once_cell::sync::OnceCell;
use parking_lot::FairMutex;
use tokio::task::spawn_blocking;

use crate::{Cell, Error, Result};

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
    gfx_freq: Cell<u32>,
    gfx_max_freq: Cell<u32>,
    mem_freq: Cell<u32>,
    mem_max_freq: Cell<u32>,
    sm_freq: Cell<u32>,
    sm_max_freq: Cell<u32>,
    video_freq: Cell<u32>,
    video_max_freq: Cell<u32>,
    mem_total: Cell<u64>,
    mem_used: Cell<u64>,
    name: Cell<String>,
    power: Cell<u32>,
    power_limit: Cell<u32>,
    power_limit_max: Cell<u32>,
    power_limit_min: Cell<u32>,
}

impl Card {
    pub async fn available() -> bool {
        available().await
    }

    pub async fn ids() -> Result<Vec<u64>> {
        devices().await
    }

    pub fn new(id: u64) -> Self {
        let gfx_freq = Cell::default();
        let gfx_max_freq = Cell::default();
        let mem_freq = Cell::default();
        let mem_max_freq = Cell::default();
        let sm_freq = Cell::default();
        let sm_max_freq = Cell::default();
        let video_freq = Cell::default();
        let video_max_freq = Cell::default();
        let mem_total = Cell::default();
        let mem_used = Cell::default();
        let name = Cell::default();
        let power = Cell::default();
        let power_limit = Cell::default();
        let power_limit_max = Cell::default();
        let power_limit_min = Cell::default();
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
        self.gfx_freq.get_or_init(gfx_freq(self.id)).await
    }

    pub async fn gfx_max_freq(&self) -> Result<u32> {
        self.gfx_max_freq.get_or_init(gfx_max_freq(self.id)).await
    }

    pub async fn mem_freq(&self) -> Result<u32> {
        self.mem_freq.get_or_init(mem_freq(self.id)).await
    }

    pub async fn mem_max_freq(&self) -> Result<u32> {
        self.mem_max_freq.get_or_init(mem_max_freq(self.id)).await
    }

    pub async fn sm_freq(&self) -> Result<u32> {
        self.sm_freq.get_or_init(sm_freq(self.id)).await
    }

    pub async fn video_freq(&self) -> Result<u32> {
        self.video_freq.get_or_init(video_freq(self.id)).await
    }

    pub async fn video_max_freq(&self) -> Result<u32> {
        self.video_max_freq.get_or_init(video_max_freq(self.id)).await
    }

    pub async fn mem_total(&self) -> Result<u64> {
        self.mem_total.get_or_init(mem_total(self.id)).await
    }

    pub async fn mem_used(&self) -> Result<u64> {
        self.mem_used.get_or_init(mem_used(self.id)).await
    }

    pub async fn name(&self) -> Result<String> {
        self.name.get_or_init(name(self.id)).await
    }

    pub async fn power(&self) -> Result<u32> {
        self.power.get_or_init(power(self.id)).await
    }

    pub async fn power_limit(&self) -> Result<u32> {
        self.power_limit.get_or_init(power_limit(self.id)).await
    }

    pub async fn power_max_limit(&self) -> Result<u32> {
        self.power_limit_max
            .get_or_init(power_max_limit(self.id))
            .await
    }

    pub async fn power_min_limit(&self) -> Result<u32> {
        self.power_limit_min
            .get_or_init(power_min_limit(self.id))
            .await
    }

    pub async fn set_gfx_freq(&self, min: u32, max: u32) -> Result<()> {
        self.gfx_freq
            .clear_if_ok(set_gfx_freq(self.id, min, max))
            .await
    }

    pub async fn reset_gfx_freq(&self) -> Result<()> {
        self.gfx_freq.clear_if_ok(reset_gfx_freq(self.id)).await
    }

    pub async fn set_power_limit(&self, v: u32) -> Result<()> {
        self.power_limit.clear_if_ok(set_power_limit(self.id, v)).await
    }

    pub async fn reset_power_limit(&self) -> Result<()> {
        self.power_limit.clear_if_ok(reset_power_limit(self.id)).await
    }
}
