pub(crate) mod path {
    use std::path::PathBuf;

    use crate::cpu::path::device_attr as cpu_device_attr;
    use crate::cpufreq::path::device_attr as cpufreq_device_attr;

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

    pub(crate) fn energy_perf_bias(id: u64) -> PathBuf {
        let mut p = cpu_device_attr(id, "power");
        p.push("energy_perf_bias");
        p
    }

    pub(crate) fn energy_performance_preference(id: u64) -> PathBuf {
        cpufreq_device_attr(id, "energy_performance_preference")
    }

    pub(crate) fn energy_performance_available_preferences(id: u64) -> PathBuf {
        cpufreq_device_attr(id, "energy_performance_available_preferences")
    }
}

pub use crate::cpufreq::devices;
use crate::{sysfs, Cell, Result};

async fn available() -> bool {
    path::status().is_file()
}

pub async fn energy_perf_bias(id: u64) -> Result<u64> {
    sysfs::read_u64(&path::energy_perf_bias(id)).await
}

pub async fn energy_performance_preference(id: u64) -> Result<String> {
    sysfs::read_string(&path::energy_performance_preference(id)).await
}

pub async fn energy_performance_available_preferences(id: u64) -> Result<Vec<String>> {
    sysfs::read_string_list(&path::energy_performance_available_preferences(id), ' ').await
}

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

pub async fn set_energy_perf_bias(id: u64, v: u64) -> Result<()> {
    sysfs::write_u64(&path::energy_perf_bias(id), v).await
}

pub async fn set_energy_performance_preference(id: u64, v: &str) -> Result<()> {
    sysfs::write_string(&path::energy_performance_preference(id), v).await
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
    pub async fn available() -> bool {
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
        self.max_perf_pct.get_or_init(max_perf_pct()).await
    }

    pub async fn min_perf_pct(&self) -> Result<u64> {
        self.min_perf_pct.get_or_init(min_perf_pct()).await
    }

    pub async fn no_turbo(&self) -> Result<bool> {
        self.no_turbo.get_or_init(no_turbo()).await
    }

    pub async fn status(&self) -> Result<String> {
        self.status.get_or_init(status()).await
    }

    pub async fn turbo_pct(&self) -> Result<u64> {
        self.turbo_pct.get_or_init(turbo_pct()).await
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

#[derive(Clone, Debug)]
pub struct Cpu {
    id: u64,
    energy_perf_bias: Cell<u64>,
    energy_performance_preference: Cell<String>,
    energy_performance_available_preferences: Cell<Vec<String>>,
}

impl Cpu {
    pub async fn available() -> bool {
        available().await
    }

    pub async fn ids() -> Result<Vec<u64>> {
        devices().await
    }

    pub fn new(id: u64) -> Self {
        let energy_perf_bias = Cell::default();
        let energy_performance_preference = Cell::default();
        let energy_performance_available_preferences = Cell::default();
        Self {
            id,
            energy_perf_bias,
            energy_performance_preference,
            energy_performance_available_preferences,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub async fn clear(&self) {
        tokio::join!(
            self.energy_perf_bias.clear(),
            self.energy_performance_preference.clear(),
            self.energy_performance_available_preferences.clear(),
        );
    }

    pub async fn energy_perf_bias(&self) -> Result<u64> {
        self.energy_perf_bias
            .get_or_init(energy_perf_bias(self.id))
            .await
    }

    pub async fn energy_performance_preference(&self) -> Result<String> {
        self.energy_performance_preference
            .get_or_init(energy_performance_preference(self.id))
            .await
    }

    pub async fn energy_performance_available_preferences(&self) -> Result<Vec<String>> {
        self.energy_performance_available_preferences
            .get_or_init(energy_performance_available_preferences(self.id))
            .await
    }

    pub async fn set_energy_perf_bias(&self, v: u64) -> Result<()> {
        self.energy_perf_bias
            .clear_if_ok(set_energy_perf_bias(self.id, v))
            .await
    }

    pub async fn set_energy_performance_preference(&self, v: impl AsRef<str>) -> Result<()> {
        self.energy_performance_preference
            .clear_if_ok(set_energy_performance_preference(self.id, v.as_ref()))
            .await
    }
}
