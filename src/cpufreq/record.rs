use std::ops::Deref;

use futures::Future;

use crate::cpufreq;
use crate::util::stream::prelude::*;
use crate::Result;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Record {
    id: u64,
    cpuinfo_max_freq: Option<u64>,
    cpuinfo_min_freq: Option<u64>,
    scaling_cur_freq: Option<u64>,
    scaling_driver: Option<String>,
    scaling_governor: Option<String>,
    scaling_available_governors: Option<Vec<String>>,
    scaling_max_freq: Option<u64>,
    scaling_min_freq: Option<u64>,
}

impl Record {
    pub fn available() -> impl Future<Output=Result<bool>> {
        cpufreq::available()
    }

    pub fn exists(id: u64) -> impl Future<Output=Result<bool>> {
        cpufreq::exists(id)
    }

    pub fn ids() -> impl Stream<Item=Result<u64>> {
        cpufreq::ids()
    }

    pub async fn load(id: u64) -> Self {
        let mut s = Self::new(id);
        s.read().await;
        s
    }

    pub fn new(id: u64) -> Self {
        Self {
            id,
            cpuinfo_max_freq: None,
            cpuinfo_min_freq: None,
            scaling_cur_freq: None,
            scaling_driver: None,
            scaling_governor: None,
            scaling_available_governors: None,
            scaling_max_freq: None,
            scaling_min_freq: None,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn is_empty(&self) -> bool {
        self == &Self::new(self.id)
    }

    pub async fn read(&mut self) -> bool {
        self.cpuinfo_max_freq = cpufreq::cpuinfo_max_freq(self.id).await.ok();
        self.cpuinfo_min_freq = cpufreq::cpuinfo_min_freq(self.id).await.ok();
        self.scaling_cur_freq = cpufreq::scaling_cur_freq(self.id).await.ok();
        self.scaling_driver = cpufreq::scaling_driver(self.id).await.ok();
        self.scaling_governor = cpufreq::scaling_governor(self.id).await.ok();
        self.scaling_available_governors = cpufreq::scaling_available_governors(self.id).await.ok();
        self.scaling_max_freq = cpufreq::scaling_max_freq(self.id).await.ok();
        self.scaling_min_freq = cpufreq::scaling_min_freq(self.id).await.ok();
        !self.is_empty()
    }

    pub fn cpuinfo_max_freq(&self) -> Option<u64> {
        self.cpuinfo_max_freq
    }

    pub fn cpuinfo_min_freq(&self) -> Option<u64> {
        self.cpuinfo_min_freq
    }

    pub fn scaling_cur_freq(&self) -> Option<u64> {
        self.scaling_cur_freq
    }

    pub fn scaling_driver(&self) -> Option<&str> {
        self.scaling_driver.as_deref()
    }

    pub fn scaling_governor(&self) -> Option<&str> {
        self.scaling_governor.as_deref()
    }

    pub fn scaling_available_governors(&self) -> Option<impl IntoIterator<Item=&str>> {
        self.scaling_available_governors
            .as_ref()
            .map(|v| v
                .iter()
                .map(Deref::deref)
                .collect::<Vec<_>>())
    }

    pub fn scaling_max_freq(&self) -> Option<u64> {
        self.scaling_max_freq
    }

    pub fn scaling_min_freq(&self) -> Option<u64> {
        self.scaling_min_freq
    }
}
