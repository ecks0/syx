pub(crate) mod path {
    use std::path::PathBuf;

    pub(crate) fn device_attr(id: (u64, Option<u64>, u64), a: &str) -> PathBuf {
        use crate::rapl::path::device_attr;
        device_attr(id.0, id.1, &format!("constraint_{}_{}", id.2, a))
    }

    pub(crate) fn name(id: (u64, Option<u64>, u64)) -> PathBuf {
        device_attr(id, "name")
    }

    pub(crate) fn max_power_uw(id: (u64, Option<u64>, u64)) -> PathBuf {
        device_attr(id, "max_power_uw")
    }

    pub(crate) fn power_limit_uw(id: (u64, Option<u64>, u64)) -> PathBuf {
        device_attr(id, "power_limit_uw")
    }

    pub(crate) fn time_window_us(id: (u64, Option<u64>, u64)) -> PathBuf {
        device_attr(id, "time_window_us")
    }
}

pub use crate::rapl::available;
use crate::rapl::zone::{Id as ZoneId, ids as zone_ids};
use crate::util::sysfs;
use crate::util::cell::Cell;
use crate::Result;

pub const LONG_TERM: &str = "long_term";
pub const SHORT_TERM: &str = "short_term";

pub async fn ids_for_zone(zone: (u64, Option<u64>)) -> Result<Vec<(u64, Option<u64>, u64)>> {
    let mut ids = vec![];
    for i in 0.. {
        let id = (zone.0, zone.1, i);
        if path::name(id).is_file() {
            ids.push(id);
        } else {
            break;
        }
    }
    Ok(ids)
}

pub async fn id_for_name(zone: (u64, Option<u64>), name_: &str) -> Result<Option<(u64, Option<u64>, u64)>> {
    for id in ids_for_zone(zone).await? {
        if name_ == name(id).await?.as_str() {
            return Ok(Some(id));
        }
    }
    Ok(None)
}

pub async fn ids() -> Result<Vec<(u64, Option<u64>, u64)>> {
    let mut ids = vec![];
    for zone in zone_ids().await? {
        let v = ids_for_zone(zone).await?;
        ids.extend(v);
    }
    Ok(ids)
}

pub async fn exists(id: (u64, Option<u64>, u64)) -> bool {
    path::name(id).is_file()
}

pub async fn name(id: (u64, Option<u64>, u64)) -> Result<String> {
    sysfs::read_string(&path::name(id)).await
}

pub async fn max_power_uw(id: (u64, Option<u64>, u64)) -> Result<u64> {
    sysfs::read_u64(&path::max_power_uw(id)).await
}

pub async fn power_limit_uw(id: (u64, Option<u64>, u64)) -> Result<u64> {
    sysfs::read_u64(&path::power_limit_uw(id)).await
}

pub async fn time_window_us(id: (u64, Option<u64>, u64)) -> Result<u64> {
    sysfs::read_u64(&path::time_window_us(id)).await
}

pub async fn set_power_limit_uw(id: (u64, Option<u64>, u64), v: u64) -> Result<()> {
    sysfs::write_u64(&path::power_limit_uw(id), v).await
}

pub async fn set_time_window_us(id: (u64, Option<u64>, u64), v: u64) -> Result<()> {
    sysfs::write_u64(&path::time_window_us(id), v).await
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id {
    package: u64,
    subzone: Option<u64>,
    index: u64,
}

impl Id {
    pub fn new(package: u64, subzone: Option<u64>, index: u64) -> Self {
        Self { package, subzone, index }
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
}

impl From<(u64, Option<u64>, u64)> for Id {
    fn from(v: (u64, Option<u64>, u64)) -> Self {
        Self::new(v.0, v.1, v.2)
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

#[derive(Clone, Debug)]
pub struct Constraint {
    id: Id,
    name: Cell<String>,
    max_power_uw: Cell<u64>,
    power_limit_uw: Cell<u64>,
    time_window_us: Cell<u64>,
}

impl Constraint {
    pub async fn available() -> bool {
        available().await
    }

    pub async fn ids() -> Result<Vec<Id>> {
        ids().await.map(|ids| ids
            .into_iter()
            .map(Id::from)
            .collect())
    }

    pub async fn ids_for_zone(zone: impl Into<ZoneId>) -> Result<Vec<Id>> {
        let zone = zone.into();
        ids_for_zone(zone.into()).await.map(|ids| ids
            .into_iter()
            .map(Id::from)
            .collect())
    }

    pub async fn id_for_name(zone: impl Into<ZoneId>, name: &str) -> Result<Option<Id>> {
        let zone = zone.into();
        id_for_name(zone.into(), name).await
            .map(|o| o.map(Id::from))
    }

    pub fn new(id: impl Into<Id>) -> Self {
        let id = id.into();
        let name = Cell::default();
        let max_power_uw = Cell::default();
        let power_limit_uw = Cell::default();
        let time_window_us = Cell::default();
        Self {
            id,
            name,
            max_power_uw,
            power_limit_uw,
            time_window_us,
        }
    }

    pub async fn clear(&self) {
        tokio::join!(
            self.name.clear(),
            self.max_power_uw.clear(),
            self.power_limit_uw.clear(),
            self.time_window_us.clear(),
        );
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub async fn name(&self) -> Result<String> {
        self.name
            .get_or_load(name(self.id.into()))
            .await
    }

    pub async fn max_power_uw(&self) -> Result<u64> {
        self.max_power_uw
            .get_or_load(max_power_uw(self.id.into()))
            .await
    }

    pub async fn power_limit_uw(&self) -> Result<u64> {
        self.power_limit_uw
            .get_or_load(power_limit_uw(self.id.into()))
            .await
    }

    pub async fn time_window_us(&self) -> Result<u64> {
        self.time_window_us
            .get_or_load(time_window_us(self.id.into()))
            .await
    }

    pub async fn set_power_limit_uw(&self, v: u64) -> Result<()> {
        let f = set_power_limit_uw(self.id.into(), v);
        self.power_limit_uw.clear_if_ok(f).await
    }

    pub async fn set_time_window_us(&self, v: u64) -> Result<()> {
        let f = set_time_window_us(self.id.into(), v);
        self.time_window_us.clear_if_ok(f).await
    }
}
