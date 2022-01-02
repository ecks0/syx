mod cache;
pub(crate) mod path;
mod values;

use async_stream::try_stream;
use futures::stream::{Stream, TryStreamExt as _};

pub use crate::drm::cache::Cache;
pub use crate::drm::values::Values;
use crate::util::sysfs;
use crate::{BusId, Error, Result};

pub async fn available() -> Result<bool> {
    Ok(path::root().is_dir())
}

pub async fn exists(id: u64) -> Result<bool> {
    Ok(path::card(id).is_dir())
}

pub fn ids() -> impl Stream<Item = Result<u64>> {
    sysfs::read_ids(&path::root(), "card")
}

pub fn ids_for_driver(driver_: impl Into<String>) -> impl Stream<Item = Result<u64>> {
    let driver_ = driver_.into();
    try_stream! {
        for await id in ids() {
            let id = id?;
            if driver_ == driver(id).await? {
                yield id;
            }
        }
    }
}

pub async fn bus_id(index: u64) -> Result<BusId> {
    let bus = sysfs::read_link_name(&path::subsystem(index)).await?;
    let id = sysfs::read_link_name(&path::device(index)).await?;
    let r = BusId { bus, id };
    Ok(r)
}

pub async fn index(bus_id: &BusId) -> Result<u64> {
    let path = path::bus_drm(bus_id);
    let indices: Vec<_> = sysfs::read_ids(&path, "card").try_collect().await?;
    if indices.is_empty() {
        let s = format!(
            "Drm card node not found for {} device {}",
            bus_id.bus, bus_id.id
        );
        #[cfg(feature = "logging")]
        log::error!("ERR {}", s);
        Err(Error::non_sequitor(s))
    } else if indices.len() > 1 {
        let s = format!(
            "Multiple drm card nodes found for {} device {}",
            bus_id.bus, bus_id.id
        );
        #[cfg(feature = "logging")]
        log::error!("ERR {}", s);
        Err(Error::non_sequitor(s))
    } else {
        Ok(indices[0])
    }
}

pub async fn driver(index: u64) -> Result<String> {
    sysfs::read_link_name(&path::driver(index)).await
}
