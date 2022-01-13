#[cfg(feature = "cache")]
mod cache;
pub(crate) mod path;
mod values;

use async_stream::stream;
use futures::pin_mut;
use futures::stream::{Stream, TryStreamExt as _};

pub use crate::intel_rapl::available;
#[cfg(feature = "cache")]
pub use crate::intel_rapl::constraint::cache::Cache;
pub use crate::intel_rapl::constraint::values::Values;
use crate::intel_rapl::zone::{ids as zone_ids, Id as ZoneId};
use crate::util::sysfs;
use crate::Result;

pub const LONG_TERM: &str = "long_term";
pub const SHORT_TERM: &str = "short_term";

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id {
    package: u64,
    subzone: Option<u64>,
    index: u64,
}

impl Id {
    pub fn new(package: u64, subzone: Option<u64>, index: u64) -> Self {
        Self {
            package,
            subzone,
            index,
        }
    }

    pub fn package(&self) -> u64 {
        self.package
    }

    pub fn subzone(&self) -> Option<u64> {
        self.subzone
    }

    pub fn index(&self) -> u64 {
        self.index
    }

    pub fn is_in(&self, zone: impl Into<ZoneId>) -> bool {
        let zone = zone.into();
        self.package == zone.package() && self.subzone == zone.subzone()
    }
}

impl From<(u64, Option<u64>, u64)> for Id {
    fn from(v: (u64, Option<u64>, u64)) -> Self {
        Self::new(v.0, v.1, v.2)
    }
}

impl From<(u64, u64, u64)> for Id {
    fn from(v: (u64, u64, u64)) -> Self {
        Self::new(v.0, Some(v.1), v.2)
    }
}

impl From<(ZoneId, u64)> for Id {
    fn from(v: (ZoneId, u64)) -> Self {
        Self::new(v.0.package(), v.0.subzone(), v.1)
    }
}

impl From<Id> for (u64, Option<u64>, u64) {
    fn from(v: Id) -> Self {
        (v.package, v.subzone, v.index)
    }
}

impl From<Id> for ZoneId {
    fn from(v: Id) -> Self {
        ZoneId::new(v.package, v.subzone)
    }
}

pub fn ids() -> impl Stream<Item = Result<Id>> {
    zone_ids().map_ok(ids_for_zone).try_flatten()
}

pub fn ids_for_zone(zone: impl Into<ZoneId>) -> impl Stream<Item = Result<Id>> {
    let zone = zone.into();
    stream! {
        let zone = zone.into();
        for c in 0.. {
            let id = Id::from((zone, c));
            if path::name(id).is_file() {
                yield Ok(id);
            } else {
                break;
            }
        }
    }
}

pub async fn id_for_name<Z, S>(zone: Z, name_: S) -> Result<Option<Id>>
where
    Z: Into<ZoneId>,
    S: Into<String>,
{
    let (zone, name_) = (zone.into(), name_.into());
    let mut r = None;
    let s = ids_for_zone(zone);
    pin_mut!(s);
    while let Some(v) = s.try_next().await? {
        if name_ == name(v).await? {
            r = Some(v);
            break;
        }
    }
    Ok(r)
}

pub async fn exists(id: impl Into<Id>) -> Result<bool> {
    Ok(path::name(id.into()).is_file())
}

pub async fn name(id: impl Into<Id>) -> Result<String> {
    sysfs::read_string(&path::name(id.into())).await
}

pub async fn max_power_uw(id: impl Into<Id>) -> Result<u64> {
    sysfs::read_u64(&path::max_power_uw(id.into())).await
}

pub async fn power_limit_uw(id: impl Into<Id>) -> Result<u64> {
    sysfs::read_u64(&path::power_limit_uw(id.into())).await
}

pub async fn time_window_us(id: impl Into<Id>) -> Result<u64> {
    sysfs::read_u64(&path::time_window_us(id.into())).await
}

pub async fn set_power_limit_uw(id: impl Into<Id>, v: u64) -> Result<()> {
    sysfs::write_u64(&path::power_limit_uw(id.into()), v).await
}

pub async fn set_time_window_us(id: impl Into<Id>, v: u64) -> Result<()> {
    sysfs::write_u64(&path::time_window_us(id.into()), v).await
}
