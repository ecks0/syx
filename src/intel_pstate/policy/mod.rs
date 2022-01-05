pub mod cache;
pub(crate) mod path;
pub mod values;

pub use crate::cpufreq::{exists, ids};
pub use crate::intel_pstate::available;
pub use crate::intel_pstate::policy::cache::Cache;
pub use crate::intel_pstate::policy::values::Values;
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
