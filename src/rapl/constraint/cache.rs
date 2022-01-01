use futures::stream::{Stream, TryStreamExt as _};
use futures::Future;

pub use crate::rapl::available;
use crate::rapl::constraint::{self, Id};
use crate::rapl::zone;
use crate::util::cell::Cached;
use crate::Result;

#[derive(Clone, Debug)]
pub struct Cache {
    id: Id,
    name: Cached<String>,
    max_power_uw: Cached<u64>,
    power_limit_uw: Cached<u64>,
    time_window_us: Cached<u64>,
}

impl Cache {
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

    pub async fn all_for_zone() -> impl Stream<Item = Result<Self>> {
        constraint::ids().map_ok(Self::new)
    }

    pub fn new(id: impl Into<Id>) -> Self {
        Self {
            id: id.into(),
            name: Cached::default(),
            max_power_uw: Cached::default(),
            power_limit_uw: Cached::default(),
            time_window_us: Cached::default(),
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
