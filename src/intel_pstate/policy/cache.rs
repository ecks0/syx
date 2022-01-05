use futures::stream::{Stream, TryStreamExt as _};
use futures::Future;

use crate::intel_pstate::policy::{self, Values};
use crate::util::cell::Cell;
use crate::Result;

#[derive(Clone, Debug)]
pub struct Cache {
    id: u64,
    energy_perf_bias: Cell<u64>,
    energy_performance_preference: Cell<String>,
    energy_performance_available_preferences: Cell<Vec<String>>,
}

impl Cache {
    pub fn available() -> impl Future<Output = Result<bool>> {
        policy::available()
    }

    pub fn exists(id: u64) -> impl Future<Output = Result<bool>> {
        policy::exists(id)
    }

    pub fn ids() -> impl Stream<Item = Result<u64>> {
        policy::ids()
    }

    pub fn all() -> impl Stream<Item = Result<Self>> {
        policy::ids().map_ok(Self::new)
    }

    pub fn new(id: u64) -> Self {
        Self {
            id,
            energy_perf_bias: Cell::default(),
            energy_performance_preference: Cell::default(),
            energy_performance_available_preferences: Cell::default(),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub async fn clear(&self) {
        tokio::join!(
            self.energy_perf_bias.clear(),
            self.energy_performance_preference.clear(),
            self.energy_performance_available_preferences.clear(),
        );
    }

    pub async fn energy_perf_bias(&self) -> Result<u64> {
        self.energy_perf_bias
            .get_or_load(policy::energy_perf_bias(self.id))
            .await
    }

    pub async fn energy_performance_preference(&self) -> Result<String> {
        self.energy_performance_preference
            .get_or_load(policy::energy_performance_preference(self.id))
            .await
    }

    pub async fn energy_performance_available_preferences(&self) -> Result<Vec<String>> {
        self.energy_performance_available_preferences
            .get_or_load(policy::energy_performance_available_preferences(self.id))
            .await
    }

    pub async fn set_energy_perf_bias(&self, v: u64) -> Result<()> {
        self.energy_perf_bias
            .clear_if_ok(policy::set_energy_perf_bias(self.id, v))
            .await
    }

    pub async fn set_energy_performance_preference(&self, v: impl AsRef<str>) -> Result<()> {
        self.energy_performance_preference
            .clear_if_ok(policy::set_energy_performance_preference(
                self.id,
                v.as_ref(),
            ))
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
