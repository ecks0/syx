use futures::Future;

use crate::pstate::system::{self, Cache};
use crate::Result;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Values;

impl Values {
    pub fn available() -> impl Future<Output = Result<bool>> {
        system::available()
    }

    pub fn max_perf_pct(&self) -> impl Future<Output = Result<u64>> {
        system::max_perf_pct()
    }

    pub fn min_perf_pct(&self) -> impl Future<Output = Result<u64>> {
        system::min_perf_pct()
    }

    pub fn no_turbo(&self) -> impl Future<Output = Result<bool>> {
        system::no_turbo()
    }

    pub fn status(&self) -> impl Future<Output = Result<String>> {
        system::status()
    }

    pub async fn is_active(&self) -> Result<bool> {
        self.status().await.map(|v| v == "active")
    }

    pub fn turbo_pct(&self) -> impl Future<Output = Result<u64>> {
        system::turbo_pct()
    }

    pub fn set_max_perf_pct(&self, v: u64) -> impl Future<Output = Result<()>> {
        system::set_max_perf_pct(v)
    }

    pub fn set_min_perf_pct(&self, v: u64) -> impl Future<Output = Result<()>> {
        system::set_min_perf_pct(v)
    }

    pub fn set_no_turbo(&self, v: bool) -> impl Future<Output = Result<()>> {
        system::set_no_turbo(v)
    }
}

impl From<Cache> for Values {
    fn from(_: Cache) -> Self {
        Self::default()
    }
}

impl From<&Cache> for Values {
    fn from(_: &Cache) -> Self {
        Self::default()
    }
}
