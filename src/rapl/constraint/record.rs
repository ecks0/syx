use futures::stream::{Stream, TryStreamExt as _};
use futures::Future;

use crate::rapl::constraint::{self, Id};
use crate::rapl::zone;
use crate::Result;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Record {
    id: Id,
    name: Option<String>,
    max_power_uw: Option<u64>,
    power_limit_uw: Option<u64>,
    time_window_us: Option<u64>,
}

impl Record {
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

    pub async fn load(id: impl Into<Id>) -> Self {
        let mut s = Self::new(id);
        s.read().await;
        s
    }

    pub fn all() -> impl Stream<Item = Result<Self>> {
        constraint::ids().and_then(|id| async move { Ok(Self::load(id).await) })
    }

    pub fn all_for_zone(zone: impl Into<zone::Id>) -> impl Stream<Item = Result<Self>> {
        constraint::ids_for_zone(zone).and_then(|id| async move { Ok(Self::load(id).await) })
    }

    pub fn new(id: impl Into<Id>) -> Self {
        Self {
            id: id.into(),
            name: None,
            max_power_uw: None,
            power_limit_uw: None,
            time_window_us: None,
        }
    }

    pub async fn read(&mut self) {
        self.name = constraint::name(self.id).await.ok();
        self.max_power_uw = constraint::max_power_uw(self.id).await.ok();
        self.power_limit_uw = constraint::power_limit_uw(self.id).await.ok();
        self.time_window_us = constraint::time_window_us(self.id).await.ok();
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub async fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub async fn max_power_uw(&self) -> Option<u64> {
        self.max_power_uw
    }

    pub async fn power_limit_uw(&self) -> Option<u64> {
        self.power_limit_uw
    }

    pub async fn time_window_us(&self) -> Option<u64> {
        self.time_window_us
    }
}
