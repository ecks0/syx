use futures::stream::{Stream, TryStreamExt as _};
use futures::Future;

use crate::pstate::policy::{self, Cache};
use crate::Result;

#[derive(Clone, Debug)]
pub struct Values {
    id: u64,
}

impl Values {
    pub fn available() -> impl Future<Output = Result<bool>> {
        policy::available()
    }

    pub fn exists(id: u64) -> impl Future<Output = Result<bool>> {
        policy::exists(id)
    }

    pub fn ids() -> impl Stream<Item = Result<u64>> {
        policy::ids()
    }

    pub fn all() -> impl Stream<Item = Result<Self>> {
        policy::ids().map_ok(Self::new)
    }

    pub fn new(id: u64) -> Self {
        Self { id }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn energy_perf_bias(&self) -> impl Future<Output=Result<u64>> {
        policy::energy_perf_bias(self.id)
    }

    pub fn energy_performance_preference(&self) -> impl Future<Output=Result<String>> {
        policy::energy_performance_preference(self.id)
    }

    pub fn energy_performance_available_preferences(&self) -> impl Future<Output=Result<Vec<String>>> {
        policy::energy_performance_available_preferences(self.id)
    }

    pub fn set_energy_perf_bias(&self, v: u64) -> impl Future<Output=Result<()>> {
        policy::set_energy_perf_bias(self.id, v)
    }

    pub async fn set_energy_performance_preference(&self, v: impl AsRef<str>) -> Result<()> {
        policy::set_energy_performance_preference(self.id, v.as_ref()).await
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
