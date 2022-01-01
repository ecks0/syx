use futures::stream::{Stream, TryStreamExt as _};
use futures::Future;

use crate::util::cell::Cached;
use crate::{drm, BusId, Result};

#[derive(Clone, Debug)]
pub struct Cache {
    id: u64,
    bus_id: Cached<BusId>,
    driver: Cached<String>,
}

impl Cache {
    pub fn available() -> impl Future<Output = Result<bool>> {
        drm::available()
    }

    pub fn exists(id: u64) -> impl Future<Output = Result<bool>> {
        drm::exists(id)
    }

    pub fn ids() -> impl Stream<Item = Result<u64>> {
        drm::ids()
    }

    pub fn all() -> impl Stream<Item = Result<Self>> {
        drm::ids().map_ok(Self::new)
    }

    pub fn new(id: u64) -> Self {
        Self {
            id,
            bus_id: Cached::default(),
            driver: Cached::default(),
        }
    }

    pub async fn clear(&self) {
        tokio::join!(self.bus_id.clear(), self.driver.clear());
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub async fn bus_id(&self) -> Result<BusId> {
        self.bus_id.get_or_load(drm::bus_id(self.id)).await
    }

    pub async fn driver(&self) -> Result<String> {
        self.driver.get_or_load(drm::driver(self.id)).await
    }
}
