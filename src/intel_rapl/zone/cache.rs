use futures::stream::{Stream, TryStreamExt as _};
use futures::Future;

use crate::intel_rapl::zone::{self, Id, Values};
use crate::util::cell::Cell;
use crate::Result;

#[derive(Clone, Debug)]
pub struct Cache {
    id: Id,
    enabled: Cell<bool>,
    energy_uj: Cell<u64>,
    max_energy_range_uj: Cell<u64>,
    name: Cell<String>,
}

impl Cache {
    pub fn available() -> impl Future<Output = Result<bool>> {
        zone::available()
    }

    pub fn exists(id: Id) -> impl Future<Output = Result<bool>> {
        zone::exists(id)
    }

    pub fn ids() -> impl Stream<Item = Result<Id>> {
        zone::ids()
    }

    pub fn all() -> impl Stream<Item = Result<Self>> {
        zone::ids().map_ok(Self::new)
    }

    pub fn new(id: impl Into<Id>) -> Self {
        Self {
            id: id.into(),
            enabled: Cell::default(),
            energy_uj: Cell::default(),
            max_energy_range_uj: Cell::default(),
            name: Cell::default(),
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
        self.enabled
            .clear_if_ok(zone::set_enabled(self.id, v))
            .await
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
