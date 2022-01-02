mod cache;
pub(crate) mod path;
mod values;

use futures::stream::Stream;

pub use crate::cpufreq::cache::Cache;
pub use crate::cpufreq::values::Values;
use crate::util::sysfs;
use crate::Result;

pub async fn available() -> Result<bool> {
    Ok(path::root().is_dir())
}

pub async fn exists(id: u64) -> Result<bool> {
    Ok(path::policy(id).is_dir())
}

pub fn ids() -> impl Stream<Item = Result<u64>> {
    sysfs::read_ids(&path::root(), "policy")
}

pub async fn cpuinfo_max_freq(id: u64) -> Result<u64> {
    sysfs::read_u64(&path::cpuinfo_max_freq(id)).await
}

pub async fn cpuinfo_min_freq(id: u64) -> Result<u64> {
    sysfs::read_u64(&path::cpuinfo_min_freq(id)).await
}

pub async fn scaling_cur_freq(id: u64) -> Result<u64> {
    sysfs::read_u64(&path::scaling_cur_freq(id)).await
}

pub async fn scaling_driver(id: u64) -> Result<String> {
    sysfs::read_string(&path::scaling_driver(id)).await
}

pub async fn scaling_governor(id: u64) -> Result<String> {
    sysfs::read_string(&path::scaling_governor(id)).await
}

pub async fn scaling_available_governors(id: u64) -> Result<Vec<String>> {
    sysfs::read_string_list(&path::scaling_available_governors(id), ' ').await
}

pub async fn scaling_max_freq(id: u64) -> Result<u64> {
    sysfs::read_u64(&path::scaling_max_freq(id)).await
}

pub async fn scaling_min_freq(id: u64) -> Result<u64> {
    sysfs::read_u64(&path::scaling_min_freq(id)).await
}

pub async fn set_scaling_governor(id: u64, v: &str) -> Result<()> {
    sysfs::write_string(&path::scaling_governor(id), v).await
}

pub async fn set_scaling_max_freq(id: u64, v: u64) -> Result<()> {
    sysfs::write_u64(&path::scaling_max_freq(id), v).await
}

pub async fn set_scaling_min_freq(id: u64, v: u64) -> Result<()> {
    sysfs::write_u64(&path::scaling_min_freq(id), v).await
}
