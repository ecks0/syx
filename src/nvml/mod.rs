mod cache;
mod values;

use std::result::Result as StdResult;
use std::sync::Arc;

use futures::stream::Stream;
use nvml_wrapper::enum_wrappers::device::{Clock as NVMLClock, ClockId as NVMLClockId};
use nvml_wrapper::error::NvmlError;
use nvml_wrapper::NVML;
use once_cell::sync::OnceCell;
use parking_lot::FairMutex;
use tokio::task::spawn_blocking;

pub use crate::nvml::cache::Cache;
pub use crate::nvml::values::Values;
use crate::{drm, Error, Result};

fn nvml() -> Result<Arc<FairMutex<NVML>>> {
    static INSTANCE: OnceCell<StdResult<Arc<FairMutex<NVML>>, NvmlError>> = OnceCell::new();
    let r = INSTANCE.get_or_init(|| NVML::init().map(|nvml| Arc::new(FairMutex::new(nvml))));
    match r {
        Ok(v) => Ok(Arc::clone(v)),
        Err(e) => Err(Error::nvml_init(e)),
    }
}

fn read_device_nvml<F, T>(bus_id: &str, name: &'static str, f: F) -> Result<T>
where
    T: std::fmt::Debug,
    F: FnOnce(&nvml_wrapper::Device) -> StdResult<T, NvmlError>,
{
    let res = {
        let nvml = nvml()?;
        let nvml = nvml.lock();
        let device = nvml
            .device_by_pci_bus_id(bus_id)
            .map_err(|e| Error::nvml_read(e, bus_id, name))?;
        let r = f(&device);
        drop(nvml);
        r.map_err(|e| Error::nvml_read(e, bus_id, name))
    };
    #[cfg(feature = "logging")]
    match &res {
        Ok(v) => log::debug!("OK nvml r {} {} {:?}", name, bus_id, v),
        Err(e) => log::warn!("ERR nvml r {} {} {}", name, bus_id, e),
    }
    res
}

fn write_device_nvml<F, T>(bus_id: &str, name: &'static str, f: F) -> Result<T>
where
    F: FnOnce(&mut nvml_wrapper::Device) -> StdResult<T, NvmlError>,
{
    let res = {
        let nvml = nvml()?;
        let nvml = nvml.lock();
        let mut device = nvml
            .device_by_pci_bus_id(bus_id)
            .map_err(|e| Error::nvml_write(e, bus_id, name))?;
        let r = f(&mut device);
        drop(nvml);
        r.map_err(|e| Error::nvml_write(e, bus_id, name))
    };
    #[cfg(feature = "logging")]
    match &res {
        Ok(_) => log::debug!("OK nvml w {} {}", name, bus_id),
        Err(e) => log::error!("ERR nvml w {} {} {}", name, bus_id, e),
    }
    res
}

async fn nvml_bus_id(id: u64) -> Result<String> {
    let bus_id = drm::bus_id(id).await?;
    let bus_id = format!("0000{}", bus_id.id);
    Ok(bus_id)
}

async fn read_device<F, T>(id: u64, name: &'static str, f: F) -> Result<T>
where
    T: std::fmt::Debug + Send + 'static,
    F: FnOnce(&nvml_wrapper::Device) -> StdResult<T, NvmlError> + Send + 'static,
{
    let id = nvml_bus_id(id).await?;
    spawn_blocking(move || read_device_nvml(&id, name, f))
        .await
        .unwrap()
}

async fn write_device<F, T>(id: u64, name: &'static str, f: F) -> Result<T>
where
    T: std::fmt::Debug + Send + 'static,
    F: FnOnce(&mut nvml_wrapper::Device) -> StdResult<T, NvmlError> + Send + 'static,
{
    let id = nvml_bus_id(id).await?;
    spawn_blocking(move || write_device_nvml(&id, name, f))
        .await
        .unwrap()
}

pub async fn available() -> Result<bool> {
    Ok(spawn_blocking(|| nvml().is_ok()).await.unwrap())
}

pub async fn exists(id: u64) -> Result<bool> {
    fn exists(bus_id: &str) -> Result<bool> {
        let nvml = nvml()?;
        let nvml = nvml.lock();
        let r = nvml.device_by_pci_bus_id(bus_id);
        Ok(r.is_ok())
    }

    let bus_id = nvml_bus_id(id).await?;
    spawn_blocking(move || exists(&bus_id)).await.unwrap()
}

pub fn ids() -> impl Stream<Item = Result<u64>> {
    drm::ids_for_driver("nvidia")
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
