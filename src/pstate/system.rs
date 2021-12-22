pub(crate) mod path {
    use std::path::PathBuf;

    pub(crate) fn root() -> PathBuf {
        PathBuf::from("/sys/devices/system/cpu/intel_pstate")
    }

    pub(crate) fn root_attr(a: &str) -> PathBuf {
        let mut p = root();
        p.push(a);
        p
    }

    pub(crate) fn max_perf_pct() -> PathBuf {
        root_attr("max_perf_pct")
    }

    pub(crate) fn min_perf_pct() -> PathBuf {
        root_attr("min_perf_pct")
    }

    pub(crate) fn no_turbo() -> PathBuf {
        root_attr("no_turbo")
    }

    pub(crate) fn status() -> PathBuf {
        root_attr("status")
    }

    pub(crate) fn turbo_pct() -> PathBuf {
        root_attr("turbo_pct")
    }
}

pub use crate::pstate::available;
use crate::util::cell::Cell;
use crate::util::sysfs;
use crate::Result;

pub async fn max_perf_pct() -> Result<u64> {
    sysfs::read_u64(&path::max_perf_pct()).await
}

pub async fn min_perf_pct() -> Result<u64> {
    sysfs::read_u64(&path::min_perf_pct()).await
}

pub async fn no_turbo() -> Result<bool> {
    sysfs::read_bool(&path::no_turbo()).await
}

pub async fn status() -> Result<String> {
    sysfs::read_string(&path::status()).await
}

pub async fn turbo_pct() -> Result<u64> {
    sysfs::read_u64(&path::turbo_pct()).await
}

pub async fn set_max_perf_pct(v: u64) -> Result<()> {
    sysfs::write_u64(&path::max_perf_pct(), v).await
}

pub async fn set_min_perf_pct(v: u64) -> Result<()> {
    sysfs::write_u64(&path::min_perf_pct(), v).await
}

pub async fn set_no_turbo(v: bool) -> Result<()> {
    sysfs::write_bool(&path::no_turbo(), v).await
}

#[derive(Clone, Debug, Default)]
pub struct System {
    max_perf_pct: Cell<u64>,
    min_perf_pct: Cell<u64>,
    no_turbo: Cell<bool>,
    status: Cell<String>,
    turbo_pct: Cell<u64>,
}

impl System {
    pub async fn available() -> Result<bool> {
        available().await
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
        self.max_perf_pct.get_or_load(max_perf_pct()).await
    }

    pub async fn min_perf_pct(&self) -> Result<u64> {
        self.min_perf_pct.get_or_load(min_perf_pct()).await
    }

    pub async fn no_turbo(&self) -> Result<bool> {
        self.no_turbo.get_or_load(no_turbo()).await
    }

    pub async fn status(&self) -> Result<String> {
        self.status.get_or_load(status()).await
    }

    pub async fn turbo_pct(&self) -> Result<u64> {
        self.turbo_pct.get_or_load(turbo_pct()).await
    }

    pub async fn set_max_perf_pct(&self, v: u64) -> Result<()> {
        self.max_perf_pct.clear_if_ok(set_max_perf_pct(v)).await
    }

    pub async fn set_min_perf_pct(&self, v: u64) -> Result<()> {
        self.min_perf_pct.clear_if_ok(set_min_perf_pct(v)).await
    }

    pub async fn set_no_turbo(&self, v: bool) -> Result<()> {
        self.no_turbo.clear_if_ok(set_no_turbo(v)).await
    }
}
