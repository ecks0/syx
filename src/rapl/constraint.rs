pub(crate) mod path {
    use std::path::PathBuf;

    use crate::rapl::constraint::Id;

    pub(crate) fn constraint_attr(id: Id, a: &str) -> PathBuf {
        use crate::rapl::path::zone_attr;
        zone_attr(id.package, id.subzone, &format!("constraint_{}_{}", id.index, a))
    }

    pub(crate) fn name(id: Id) -> PathBuf {
        constraint_attr(id, "name")
    }

    pub(crate) fn max_power_uw(id: Id) -> PathBuf {
        constraint_attr(id, "max_power_uw")
    }

    pub(crate) fn power_limit_uw(id: Id) -> PathBuf {
        constraint_attr(id, "power_limit_uw")
    }

    pub(crate) fn time_window_us(id: Id) -> PathBuf {
        constraint_attr(id, "time_window_us")
    }
}

pub use crate::rapl::available;
use crate::rapl::zone::{Id as ZoneId, ids as zone_ids};
use crate::util::sysfs;
use crate::util::cell::Cell;
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

pub async fn ids() -> Result<Vec<Id>> {
    let mut ids = vec![];
    for zone in zone_ids().await? {
        let v = ids_for_zone(zone).await?;
        ids.extend(v);
    }
    Ok(ids)
}

pub async fn ids_for_zone(zone: impl Into<ZoneId>) -> Result<Vec<Id>> {
    let zone = zone.into();
    let mut ids = vec![];
    for i in 0.. {
        let id = Id::from((zone, i));
        if path::name(id).is_file() {
            ids.push(id);
        } else {
            break;
        }
    }
    Ok(ids)
}

pub async fn id_for_name(zone: impl Into<ZoneId>, name_: &str) -> Result<Option<Id>> {
    for id in ids_for_zone(zone).await? {
        if name_ == name(id).await?.as_str() {
            return Ok(Some(id));
        }
    }
    Ok(None)
}

pub async fn exists(id: impl Into<Id>) -> bool {
    path::name(id.into()).is_file()
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
        ids_for_zone(zone.into()).await.map(|ids| ids
            .into_iter()
            .map(Id::from)
            .collect())
    }

    pub async fn id_for_name(zone: impl Into<ZoneId>, name: &str) -> Result<Option<Id>> {
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
            .get_or_load(name(self.id))
            .await
    }

    pub async fn max_power_uw(&self) -> Result<u64> {
        self.max_power_uw
            .get_or_load(max_power_uw(self.id))
            .await
    }

    pub async fn power_limit_uw(&self) -> Result<u64> {
        self.power_limit_uw
            .get_or_load(power_limit_uw(self.id))
            .await
    }

    pub async fn time_window_us(&self) -> Result<u64> {
        self.time_window_us
            .get_or_load(time_window_us(self.id))
            .await
    }

    pub async fn set_power_limit_uw(&self, v: u64) -> Result<()> {
        let f = set_power_limit_uw(self.id, v);
        self.power_limit_uw.clear_if_ok(f).await
    }

    pub async fn set_time_window_us(&self, v: u64) -> Result<()> {
        let f = set_time_window_us(self.id, v);
        self.time_window_us.clear_if_ok(f).await
    }
}


