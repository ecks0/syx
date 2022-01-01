use futures::stream::{Stream, TryStreamExt as _};
use futures::Future;

use crate::rapl::zone::{self, Id};
use crate::Result;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Record {
    id: Id,
    enabled: Option<bool>,
    energy_uj: Option<u64>,
    max_energy_range_uj: Option<u64>,
    name: Option<String>,
}

impl Record {
    pub fn available() -> impl Future<Output = Result<bool>> {
        zone::available()
    }

    pub fn exists(id: Id) -> impl Future<Output = Result<bool>> {
        zone::exists(id)
    }

    pub fn ids() -> impl Stream<Item = Result<Id>> {
        zone::ids()
    }

    pub async fn load(id: impl Into<Id>) -> Self {
        let mut s = Self::new(id);
        s.read().await;
        s
    }

    pub fn all() -> impl Stream<Item = Result<Self>> {
        zone::ids().and_then(|id| async move { Ok(Self::load(id).await) })
    }

    pub fn new(id: impl Into<Id>) -> Self {
        Self {
            id: id.into(),
            enabled: None,
            energy_uj: None,
            max_energy_range_uj: None,
            name: None,
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn is_empty(&self) -> bool {
        self == &Self::new(self.id)
    }

    pub async fn read(&mut self) -> bool {
        self.enabled = zone::enabled(self.id).await.ok();
        self.energy_uj = zone::energy_uj(self.id).await.ok();
        self.max_energy_range_uj = zone::max_energy_range_uj(self.id).await.ok();
        self.name = zone::name(self.id).await.ok();
        self.is_empty()
    }

    pub fn enabled(&self) -> Option<bool> {
        self.enabled
    }

    pub fn energy_uj(&self) -> Option<u64> {
        self.energy_uj
    }

    pub fn max_energy_range_uj(&self) -> Option<u64> {
        self.max_energy_range_uj
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}
