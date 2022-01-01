use futures::Future;

use crate::pstate::system;
use crate::Result;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Record {
    max_perf_pct: Option<u64>,
    min_perf_pct: Option<u64>,
    no_turbo: Option<bool>,
    status: Option<String>,
    turbo_pct: Option<u64>,
}

impl Record {
    pub fn available() -> impl Future<Output = Result<bool>> {
        system::available()
    }

    pub async fn load() -> Self {
        let mut s = Self::default();
        s.read().await;
        s
    }

    pub fn is_empty(&self) -> bool {
        self == &Self::default()
    }

    pub async fn read(&mut self) {
        self.max_perf_pct = system::max_perf_pct().await.ok();
        self.min_perf_pct = system::min_perf_pct().await.ok();
        self.no_turbo = system::no_turbo().await.ok();
        self.status = system::status().await.ok();
        self.turbo_pct = system::turbo_pct().await.ok();
    }

    pub fn max_perf_pct(&self) -> Option<u64> {
        self.max_perf_pct
    }

    pub fn min_perf_pct(&self) -> Option<u64> {
        self.min_perf_pct
    }

    pub fn no_turbo(&self) -> Option<bool> {
        self.no_turbo
    }

    pub fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }

    pub fn turbo_pct(&self) -> Option<u64> {
        self.turbo_pct
    }
}
