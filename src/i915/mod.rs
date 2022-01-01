mod cache;
pub(crate) mod path;
mod record;

use futures::stream::Stream;

pub use crate::i915::cache::Cache;
pub use crate::i915::record::Record;
use crate::util::sysfs;
use crate::{drm, Result};

pub async fn available() -> Result<bool> {
    Ok(path::module().is_dir())
}

pub async fn exists(id: u64) -> Result<bool> {
    let r = if drm::exists(id).await? {
        "i915" == drm::driver(id).await?.as_str()
    } else {
        false
    };
    Ok(r)
}

pub fn ids() -> impl Stream<Item = Result<u64>> {
    drm::ids_for_driver("i915")
}

pub async fn act_freq_mhz(id: u64) -> Result<u64> {
    sysfs::read_u64(&path::act_freq_mhz(id)).await
}

pub async fn boost_freq_mhz(id: u64) -> Result<u64> {
    sysfs::read_u64(&path::boost_freq_mhz(id)).await
}

pub async fn cur_freq_mhz(id: u64) -> Result<u64> {
    sysfs::read_u64(&path::cur_freq_mhz(id)).await
}

pub async fn max_freq_mhz(id: u64) -> Result<u64> {
    sysfs::read_u64(&path::max_freq_mhz(id)).await
}

pub async fn min_freq_mhz(id: u64) -> Result<u64> {
    sysfs::read_u64(&path::min_freq_mhz(id)).await
}

pub async fn rp0_freq_mhz(id: u64) -> Result<u64> {
    sysfs::read_u64(&path::rp0_freq_mhz(id)).await
}

pub async fn rp1_freq_mhz(id: u64) -> Result<u64> {
    sysfs::read_u64(&path::rp1_freq_mhz(id)).await
}

pub async fn rpn_freq_mhz(id: u64) -> Result<u64> {
    sysfs::read_u64(&path::rpn_freq_mhz(id)).await
}

pub async fn set_boost_freq_mhz(id: u64, v: u64) -> Result<()> {
    sysfs::write_u64(&path::boost_freq_mhz(id), v).await
}

pub async fn set_max_freq_mhz(id: u64, v: u64) -> Result<()> {
    sysfs::write_u64(&path::max_freq_mhz(id), v).await
}

pub async fn set_min_freq_mhz(id: u64, v: u64) -> Result<()> {
    sysfs::write_u64(&path::min_freq_mhz(id), v).await
}

pub async fn set_rp0_freq_mhz(id: u64, v: u64) -> Result<()> {
    sysfs::write_u64(&path::rp0_freq_mhz(id), v).await
}

pub async fn set_rp1_freq_mhz(id: u64, v: u64) -> Result<()> {
    sysfs::write_u64(&path::rp1_freq_mhz(id), v).await
}

pub async fn set_rpn_freq_mhz(id: u64, v: u64) -> Result<()> {
    sysfs::write_u64(&path::rpn_freq_mhz(id), v).await
}
