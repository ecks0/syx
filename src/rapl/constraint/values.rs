use futures::stream::{Stream, TryStreamExt as _};
use futures::Future;

pub use crate::rapl::available;
use crate::rapl::constraint::{self, Cache, Id};
use crate::rapl::zone;
use crate::Result;

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Values {
    id: Id,
}

impl Values {
    pub const LONG_TERM: &'static str = crate::rapl::constraint::LONG_TERM;
    pub const SHORT_TERM: &'static str = crate::rapl::constraint::SHORT_TERM;

    pub fn available() -> impl Future<Output = Result<bool>> {
        constraint::available()
    }

    pub fn exists(id: Id) -> impl Future<Output = Result<bool>> {
        constraint::exists(id)
    }

    pub fn ids() -> impl Stream<Item = Result<Id>> {
        constraint::ids()
    }

    pub fn ids_for_zone(zone: impl Into<zone::Id>) -> impl Stream<Item = Result<Id>> {
        constraint::ids_for_zone(zone)
    }

    pub fn id_for_name<Z, S>(zone: Z, name: S) -> impl Future<Output = Result<Option<Id>>>
    where
        Z: Into<zone::Id>,
        S: Into<String>,
    {
        constraint::id_for_name(zone, name)
    }

    pub fn all() -> impl Stream<Item = Result<Self>> {
        constraint::ids().map_ok(Self::new)
    }

    pub fn all_for_zone(zone: impl Into<zone::Id>) -> impl Stream<Item = Result<Self>> {
        constraint::ids_for_zone(zone).map_ok(Self::new)
    }

    pub async fn for_name(zone: impl Into<zone::Id>, name: &str) -> Result<Option<Self>> {
        Ok(constraint::id_for_name(zone, name).await?.map(Self::new))
    }

    pub fn new(id: impl Into<Id>) -> Self {
        Self { id: id.into() }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn name(&self) -> impl Future<Output = Result<String>> {
        constraint::name(self.id)
    }

    pub fn max_power_uw(&self) -> impl Future<Output = Result<u64>> {
        constraint::max_power_uw(self.id)
    }

    pub fn power_limit_uw(&self) -> impl Future<Output = Result<u64>> {
        constraint::power_limit_uw(self.id)
    }

    pub fn time_window_us(&self) -> impl Future<Output = Result<u64>> {
        constraint::time_window_us(self.id)
    }

    pub fn set_power_limit_uw(&self, v: u64) -> impl Future<Output = Result<()>> {
        constraint::set_power_limit_uw(self.id, v)
    }

    pub fn set_time_window_us(&self, v: u64) -> impl Future<Output = Result<()>> {
        constraint::set_time_window_us(self.id, v)
    }
}

impl From<Cache> for Values {
    fn from(v: Cache) -> Self {
        Self::new(v.id())
    }
}

impl From<&Cache> for Values {
    fn from(v: &Cache) -> Self {
        Self::new(v.id())
    }
}
