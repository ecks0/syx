use futures::Future;

use crate::cpufreq;
use crate::util::cell::Cached;
use crate::util::stream::prelude::*;
use crate::Result;

#[derive(Clone, Debug)]
pub struct Cache {
    id: u64,
    cpuinfo_max_freq: Cached<u64>,
    cpuinfo_min_freq: Cached<u64>,
    scaling_cur_freq: Cached<u64>,
    scaling_driver: Cached<String>,
    scaling_governor: Cached<String>,
    scaling_available_governors: Cached<Vec<String>>,
    scaling_max_freq: Cached<u64>,
    scaling_min_freq: Cached<u64>,
}

impl Cache {
    pub fn available() -> impl Future<Output=Result<bool>> {
        cpufreq::available()
    }

    pub fn exists(id: u64) -> impl Future<Output=Result<bool>> {
        cpufreq::exists(id)
    }

    pub fn ids() -> impl Stream<Item=Result<u64>> {
        cpufreq::ids()
    }

    pub fn new(id: u64) -> Self {
        Self {
            id,
            cpuinfo_max_freq: Cached::default(),
            cpuinfo_min_freq: Cached::default(),
            scaling_cur_freq: Cached::default(),
            scaling_driver: Cached::default(),
            scaling_governor: Cached::default(),
            scaling_available_governors: Cached::default(),
            scaling_max_freq: Cached::default(),
            scaling_min_freq: Cached::default(),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub async fn clear(&self) {
        tokio::join!(
            self.cpuinfo_max_freq.clear(),
            self.cpuinfo_min_freq.clear(),
            self.scaling_cur_freq.clear(),
            self.scaling_driver.clear(),
            self.scaling_governor.clear(),
            self.scaling_available_governors.clear(),
            self.scaling_max_freq.clear(),
            self.scaling_min_freq.clear(),
        );
    }

    pub async fn cpuinfo_max_freq(&self) -> Result<u64> {
        self.cpuinfo_max_freq
            .get_or_load(cpufreq::cpuinfo_max_freq(self.id))
            .await
    }

    pub async fn cpuinfo_min_freq(&self) -> Result<u64> {
        self.cpuinfo_min_freq
            .get_or_load(cpufreq::cpuinfo_min_freq(self.id))
            .await
    }

    pub async fn scaling_cur_freq(&self) -> Result<u64> {
        self.scaling_cur_freq
            .get_or_load(cpufreq::scaling_cur_freq(self.id))
            .await
    }

    pub async fn scaling_driver(&self) -> Result<String> {
        self.scaling_driver
            .get_or_load(cpufreq::scaling_driver(self.id))
            .await
    }

    pub async fn scaling_governor(&self) -> Result<String> {
        self.scaling_governor
            .get_or_load(cpufreq::scaling_governor(self.id))
            .await
    }

    pub async fn scaling_available_governors(&self) -> Result<Vec<String>> {
        self.scaling_available_governors
            .get_or_load(cpufreq::scaling_available_governors(self.id))
            .await
    }

    pub async fn scaling_max_freq(&self) -> Result<u64> {
        self.scaling_max_freq
            .get_or_load(cpufreq::scaling_max_freq(self.id))
            .await
    }

    pub async fn scaling_min_freq(&self) -> Result<u64> {
        self.scaling_min_freq
            .get_or_load(cpufreq::scaling_min_freq(self.id))
            .await
    }

    pub async fn set_scaling_governor(&self, v: impl AsRef<str>) -> Result<()> {
        self.scaling_governor
            .clear_if_ok(cpufreq::set_scaling_governor(self.id, v.as_ref()))
            .await
    }

    pub async fn set_scaling_max_freq(&self, v: u64) -> Result<()> {
        self.scaling_max_freq
            .clear_if_ok(cpufreq::set_scaling_max_freq(self.id, v))
            .await
    }

    pub async fn set_scaling_min_freq(&self, v: u64) -> Result<()> {
        self.scaling_min_freq
            .clear_if_ok(cpufreq::set_scaling_min_freq(self.id, v))
            .await
    }
}
