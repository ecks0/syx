pub mod cache;
pub(crate) mod path;
pub mod record;

use futures::stream::Stream;

pub use crate::cpu::cache::Cache;
pub use crate::cpu::record::Record;
use crate::util::sysfs;
use crate::Result;

pub async fn available() -> Result<bool> {
    Ok(path::root().is_dir())
}

pub async fn exists(id: u64) -> Result<bool> {
    Ok(path::cpu(id).is_dir())
}

pub fn ids() -> impl Stream<Item = Result<u64>> {
    sysfs::read_ids(&path::root(), "cpu")
}

pub async fn ids_online() -> impl Stream<Item = Result<u64>> {
    sysfs::read_indices(&path::ids_online())
}

pub async fn ids_offline() -> impl Stream<Item = Result<u64>> {
    sysfs::read_indices(&path::ids_offline())
}

pub async fn ids_present() -> impl Stream<Item = Result<u64>> {
    sysfs::read_indices(&path::ids_present())
}

pub async fn ids_possible() -> impl Stream<Item = Result<u64>> {
    sysfs::read_indices(&path::ids_possible())
}

pub async fn online(id: u64) -> Result<bool> {
    sysfs::read_bool(&path::online(id)).await
}

pub async fn set_online(id: u64, v: bool) -> Result<()> {
    sysfs::write_bool(&path::online(id), v).await
}
