use futures::stream::{Stream, TryStreamExt as _};
use futures::Future;

use crate::drm::{self, Cache};
use crate::{BusId, Result};

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Values {
    id: u64,
}

impl Values {
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
        Self { id }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub async fn bus_id(&self) -> Result<BusId> {
        drm::bus_id(self.id).await
    }

    pub async fn driver(&self) -> Result<String> {
        drm::driver(self.id).await
    }
}

impl From<Cache> for Values {
    fn from(v: Cache) -> Self {
        Self::new(v.id())
    }
}

impl From<&Cache> for Values {
    fn from(v: &Cache) -> Self {
        Self::new(v.id())
    }
}
