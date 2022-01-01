use futures::stream::{Stream, TryStreamExt as _};
use futures::Future;

use crate::{drm, BusId, Result};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Record {
    id: u64,
    bus_id: Option<BusId>,
    driver: Option<String>,
}

impl Record {
    pub fn available() -> impl Future<Output = Result<bool>> {
        drm::available()
    }

    pub fn exists(id: u64) -> impl Future<Output = Result<bool>> {
        drm::exists(id)
    }

    pub fn ids() -> impl Stream<Item = Result<u64>> {
        drm::ids()
    }

    pub async fn load(id: u64) -> Self {
        let mut s = Self::new(id);
        s.read().await;
        s
    }

    pub fn all() -> impl Stream<Item = Result<Self>> {
        drm::ids().and_then(|id| async move { Ok(Self::load(id).await) })
    }

    pub fn new(id: u64) -> Self {
        Self {
            id,
            bus_id: None,
            driver: None,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn is_empty(&self) -> bool {
        self == &Self::new(self.id)
    }

    pub async fn read(&mut self) -> bool {
        self.bus_id = drm::bus_id(self.id).await.ok();
        self.driver = drm::driver(self.id).await.ok();
        !self.is_empty()
    }

    pub async fn bus_id(&self) -> Option<&BusId> {
        self.bus_id.as_ref()
    }

    pub async fn driver(&self) -> Option<&str> {
        self.driver.as_deref()
    }
}
