pub(crate) mod path {
    use std::path::PathBuf;

    use crate::cpu::path::cpu_attr;
    use crate::cpufreq::path::policy_attr;

    pub(crate) fn energy_perf_bias(id: u64) -> PathBuf {
        let mut p = cpu_attr(id, "power");
        p.push("energy_perf_bias");
        p
    }

    pub(crate) fn energy_performance_preference(id: u64) -> PathBuf {
        policy_attr(id, "energy_performance_preference")
    }

    pub(crate) fn energy_performance_available_preferences(id: u64) -> PathBuf {
        policy_attr(id, "energy_performance_available_preferences")
    }
}

pub use crate::cpufreq::{exists, ids};
pub use crate::pstate::available;
use crate::util::cell::Cell;
use crate::util::sysfs;
use crate::Result;

pub async fn energy_perf_bias(id: u64) -> Result<u64> {
    sysfs::read_u64(&path::energy_perf_bias(id)).await
}

pub async fn energy_performance_preference(id: u64) -> Result<String> {
    sysfs::read_string(&path::energy_performance_preference(id)).await
}

pub async fn energy_performance_available_preferences(id: u64) -> Result<Vec<String>> {
    sysfs::read_string_list(&path::energy_performance_available_preferences(id), ' ').await
}

pub async fn set_energy_perf_bias(id: u64, v: u64) -> Result<()> {
    sysfs::write_u64(&path::energy_perf_bias(id), v).await
}

pub async fn set_energy_performance_preference(id: u64, v: &str) -> Result<()> {
    sysfs::write_string(&path::energy_performance_preference(id), v).await
}

#[derive(Clone, Debug)]
pub struct Policy {
    id: u64,
    energy_perf_bias: Cell<u64>,
    energy_performance_preference: Cell<String>,
    energy_performance_available_preferences: Cell<Vec<String>>,
}

impl Policy {
    pub async fn available() -> Result<bool> {
        available().await
    }

    pub async fn exists(id: u64) -> Result<bool> {
        exists(id).await
    }

    pub async fn ids() -> Result<Vec<u64>> {
        ids().await
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
            .get_or_load(energy_perf_bias(self.id))
            .await
    }

    pub async fn energy_performance_preference(&self) -> Result<String> {
        self.energy_performance_preference
            .get_or_load(energy_performance_preference(self.id))
            .await
    }

    pub async fn energy_performance_available_preferences(&self) -> Result<Vec<String>> {
        self.energy_performance_available_preferences
            .get_or_load(energy_performance_available_preferences(self.id))
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
