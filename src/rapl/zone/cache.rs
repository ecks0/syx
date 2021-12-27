use futures::Future;

use crate::rapl::zone::{self, Id};
use crate::util::cell::Cached;
use crate::util::stream::prelude::*;
use crate::Result;

#[derive(Clone, Debug)]
pub struct Cache {
    id: Id,
    enabled: Cached<bool>,
    energy_uj: Cached<u64>,
    max_energy_range_uj: Cached<u64>,
    name: Cached<String>,
}

impl Cache {
    pub fn available() -> impl Future<Output=Result<bool>> {
        zone::available()
    }

    pub fn exists(id: Id) -> impl Future<Output=Result<bool>> {
        zone::exists(id)
    }

    pub fn ids() -> impl Stream<Item=Result<Id>> {
        zone::ids()
    }

    pub fn new(id: impl Into<Id>) -> Self {
        Self {
            id: id.into(),
            enabled: Cached::default(),
            energy_uj: Cached::default(),
            max_energy_range_uj: Cached::default(),
            name: Cached::default(),
        }
    }

    pub async fn clear(&self) {
        tokio::join!(
            self.enabled.clear(),
            self.energy_uj.clear(),
            self.max_energy_range_uj.clear(),
            self.name.clear(),
        );
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub async fn enabled(&self) -> Result<bool> {
        self.enabled.get_or_load(zone::enabled(self.id)).await
    }

    pub async fn energy_uj(&self) -> Result<u64> {
        self.energy_uj.get_or_load(zone::energy_uj(self.id)).await
    }

    pub async fn max_energy_range_uj(&self) -> Result<u64> {
        self.max_energy_range_uj
            .get_or_load(zone::max_energy_range_uj(self.id))
            .await
    }

    pub async fn name(&self) -> Result<String> {
        self.name.get_or_load(zone::name(self.id)).await
    }

    pub async fn set_enabled(&self, v: bool) -> Result<()> {
        self.enabled.clear_if_ok(zone::set_enabled(self.id, v)).await
    }
}
