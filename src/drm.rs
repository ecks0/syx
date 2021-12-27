pub(crate) mod path {
    use std::path::PathBuf;

    use crate::BusId;

    pub(crate) fn root() -> PathBuf {
        PathBuf::from("/sys/class/drm")
    }

    pub(crate) fn card(id: u64) -> PathBuf {
        let mut p = root();
        p.push(format!("card{}", id));
        p
    }

    pub(crate) fn card_attr(id: u64, a: &str) -> PathBuf {
        let mut p = card(id);
        p.push(a);
        p
    }

    pub(crate) fn device(id: u64) -> PathBuf {
        card_attr(id, "device")
    }

    pub(crate) fn device_attr(id: u64, a: &str) -> PathBuf {
        let mut p = device(id);
        p.push(a);
        p
    }

    pub(crate) fn subsystem(id: u64) -> PathBuf {
        device_attr(id, "subsystem")
    }

    pub(crate) fn driver(id: u64) -> PathBuf {
        device_attr(id, "driver")
    }

    pub(crate) fn bus_drm(bus_id: &BusId) -> PathBuf {
        let s = format!("/sys/bus/{}/devices/{}/drm", bus_id.bus, bus_id.id);
        PathBuf::from(s)
    }
}

use crate::util::cell::Cached;
use crate::util::stream::prelude::*;
use crate::util::stream;
use crate::util::sysfs;
use crate::{BusId, Error, Result};

pub async fn available() -> Result<bool> {
    Ok(path::root().is_dir())
}

pub async fn exists(id: u64) -> Result<bool> {
    Ok(path::card(id).is_dir())
}

pub fn ids() -> impl Stream<Item=Result<u64>> {
    sysfs::read_ids(path::root(), "card")
}

pub fn ids_for_driver(driver_: impl Into<String>) -> impl Stream<Item=Result<u64>> {
    try_stream! {
        let driver_ = driver_.into();
        for await id in ids() {
            let id = id?;
            if driver_ == driver(id).await?.as_str() {
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
    let s = sysfs::read_ids(path::bus_drm(bus_id), "card");
    let indices: Vec<_> = stream::collect(s).await?;
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

#[derive(Clone, Debug)]
pub struct Card {
    id: u64,
    bus_id: Cached<BusId>,
    driver: Cached<String>,
}

impl Card {
    pub async fn available() -> Result<bool> {
        available().await
    }

    pub async fn ids() ->  impl Stream<Item=Result<u64>> {
        ids()
    }

    pub fn new(id: u64) -> Self {
        let bus_id = Cached::default();
        let driver = Cached::default();
        Self { id, bus_id, driver }
    }

    pub async fn clear(&self) {
        tokio::join!(self.bus_id.clear(), self.driver.clear());
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub async fn bus_id(&self) -> Result<BusId> {
        self.bus_id.get_or_load(bus_id(self.id)).await
    }

    pub async fn driver(&self) -> Result<String> {
        self.driver.get_or_load(driver(self.id)).await
    }
}
