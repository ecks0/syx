use futures::Future;

use crate::intel_pstate::system::{self, Values};
use crate::util::cell::Cell;
use crate::Result;

#[derive(Clone, Debug, Default)]
pub struct Cache {
    max_perf_pct: Cell<u64>,
    min_perf_pct: Cell<u64>,
    no_turbo: Cell<bool>,
    status: Cell<String>,
    turbo_pct: Cell<u64>,
}

impl Cache {
    pub fn available() -> impl Future<Output = Result<bool>> {
        system::available()
    }

    pub async fn clear(&self) {
        tokio::join!(
            self.max_perf_pct.clear(),
            self.min_perf_pct.clear(),
            self.no_turbo.clear(),
            self.status.clear(),
            self.turbo_pct.clear(),
        );
    }

    pub async fn max_perf_pct(&self) -> Result<u64> {
        self.max_perf_pct.get_or_load(system::max_perf_pct()).await
    }

    pub async fn min_perf_pct(&self) -> Result<u64> {
        self.min_perf_pct.get_or_load(system::min_perf_pct()).await
    }

    pub async fn no_turbo(&self) -> Result<bool> {
        self.no_turbo.get_or_load(system::no_turbo()).await
    }

    pub async fn status(&self) -> Result<String> {
        self.status.get_or_load(system::status()).await
    }

    pub async fn is_active(&self) -> Result<bool> {
        self.status().await.map(|v| v == "active")
    }

    pub async fn turbo_pct(&self) -> Result<u64> {
        self.turbo_pct.get_or_load(system::turbo_pct()).await
    }

    pub async fn set_max_perf_pct(&self, v: u64) -> Result<()> {
        self.max_perf_pct
            .clear_if_ok(system::set_max_perf_pct(v))
            .await
    }

    pub async fn set_min_perf_pct(&self, v: u64) -> Result<()> {
        self.min_perf_pct
            .clear_if_ok(system::set_min_perf_pct(v))
            .await
    }

    pub async fn set_no_turbo(&self, v: bool) -> Result<()> {
        self.no_turbo.clear_if_ok(system::set_no_turbo(v)).await
    }
}

impl From<Values> for Cache {
    fn from(_: Values) -> Self {
        Self::default()
    }
}

impl From<&Values> for Cache {
    fn from(_: &Values) -> Self {
        Self::default()
    }
}
