use futures::Future;

use crate::pstate::policy;
use crate::util::stream::prelude::*;
use crate::Result;
use std::ops::Deref;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Record {
    id: u64,
    energy_perf_bias: Option<u64>,
    energy_performance_preference: Option<String>,
    energy_performance_available_preferences: Option<Vec<String>>,
}

impl Record {
    pub fn available() -> impl Future<Output=Result<bool>> {
        policy::available()
    }

    pub fn exists(id: u64) -> impl Future<Output=Result<bool>> {
        policy::exists(id)
    }

    pub fn ids() -> impl Stream<Item=Result<u64>> {
        policy::ids()
    }

    pub async fn load(id: u64) -> Self {
        let mut s = Self::new(id);
        s.read().await;
        s
    }

    pub fn new(id: u64) -> Self {
        Self {
            id,
            energy_perf_bias: None,
            energy_performance_preference: None,
            energy_performance_available_preferences: None,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn is_empty(&self) -> bool {
        self == &Self::new(self.id)
    }

    pub async fn read(&mut self) -> bool {
        self.energy_perf_bias = policy::energy_perf_bias(self.id).await.ok();
        self.energy_performance_preference =
            policy::energy_performance_preference(self.id).await.ok();
        self.energy_performance_available_preferences =
            policy::energy_performance_available_preferences(self.id).await.ok();
        !self.is_empty()
    }

    pub fn energy_perf_bias(&self) -> Option<u64> {
        self.energy_perf_bias
    }

    pub fn energy_performance_preference(&self) -> Option<&str> {
        self.energy_performance_preference.as_deref()
    }

    pub fn energy_performance_available_preferences(&self) -> Option<impl IntoIterator<Item=&str>> {
        self.energy_performance_available_preferences
            .as_ref()
            .map(|v| v
                .iter()
                .map(Deref::deref)
                .collect::<Vec<_>>())
    }
}
