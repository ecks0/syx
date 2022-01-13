use futures::stream::{Stream, TryStreamExt as _};
use futures::Future;

use crate::intel_rapl::zone::{self, Id};
#[cfg(feature = "cache")]
use crate::intel_rapl::zone::Cache;

use crate::Result;

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Values {
    id: Id,
}

impl Values {
    pub fn available() -> impl Future<Output = Result<bool>> {
        zone::available()
    }

    pub fn exists(id: Id) -> impl Future<Output = Result<bool>> {
        zone::exists(id)
    }

    pub fn ids() -> impl Stream<Item = Result<Id>> {
        zone::ids()
    }

    pub fn all() -> impl Stream<Item = Result<Self>> {
        zone::ids().map_ok(Self::new)
    }

    pub fn new(id: impl Into<Id>) -> Self {
        Self { id: id.into() }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn enabled(&self) -> impl Future<Output = Result<bool>> {
        zone::enabled(self.id)
    }

    pub fn energy_uj(&self) -> impl Future<Output = Result<u64>> {
        zone::energy_uj(self.id)
    }

    pub fn max_energy_range_uj(&self) -> impl Future<Output = Result<u64>> {
        zone::max_energy_range_uj(self.id)
    }

    pub fn name(&self) -> impl Future<Output = Result<String>> {
        zone::name(self.id)
    }

    pub fn set_enabled(&self, v: bool) -> impl Future<Output = Result<()>> {
        zone::set_enabled(self.id, v)
    }
}

#[cfg(feature = "cache")]
impl From<Cache> for Values {
    fn from(v: Cache) -> Self {
        Self::new(v.id())
    }
}

#[cfg(feature = "cache")]
impl From<&Cache> for Values {
    fn from(v: &Cache) -> Self {
        Self::new(v.id())
    }
}
