use futures::stream::{Stream, TryStreamExt as _};
use futures::Future;

pub use crate::intel_rapl::available;
use crate::intel_rapl::constraint::{self, Id, Values};
use crate::intel_rapl::zone;
use crate::util::cell::Cell;
use crate::Result;

#[derive(Clone, Debug)]
pub struct Cache {
    id: Id,
    name: Cell<String>,
    max_power_uw: Cell<u64>,
    power_limit_uw: Cell<u64>,
    time_window_us: Cell<u64>,
}

impl Cache {
    pub const LONG_TERM: &'static str = crate::intel_rapl::constraint::LONG_TERM;
    pub const SHORT_TERM: &'static str = crate::intel_rapl::constraint::SHORT_TERM;

    pub fn available() -> impl Future<Output = Result<bool>> {
        constraint::available()
    }

    pub async fn exists(id: Id) -> impl Future<Output = Result<bool>> {
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
        Self {
            id: id.into(),
            name: Cell::default(),
            max_power_uw: Cell::default(),
            power_limit_uw: Cell::default(),
            time_window_us: Cell::default(),
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
        self.name.get_or_load(constraint::name(self.id)).await
    }

    pub async fn max_power_uw(&self) -> Result<u64> {
        self.max_power_uw
            .get_or_load(constraint::max_power_uw(self.id))
            .await
    }

    pub async fn power_limit_uw(&self) -> Result<u64> {
        self.power_limit_uw
            .get_or_load(constraint::power_limit_uw(self.id))
            .await
    }

    pub async fn time_window_us(&self) -> Result<u64> {
        self.time_window_us
            .get_or_load(constraint::time_window_us(self.id))
            .await
    }

    pub async fn set_power_limit_uw(&self, v: u64) -> Result<()> {
        let f = constraint::set_power_limit_uw(self.id, v);
        self.power_limit_uw.clear_if_ok(f).await
    }

    pub async fn set_time_window_us(&self, v: u64) -> Result<()> {
        let f = constraint::set_time_window_us(self.id, v);
        self.time_window_us.clear_if_ok(f).await
    }
}

impl From<Values> for Cache {
    fn from(v: Values) -> Self {
        Self::new(v.id())
    }
}

impl From<&Values> for Cache {
    fn from(v: &Values) -> Self {
        Self::new(v.id())
    }
}
