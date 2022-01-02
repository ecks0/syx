use futures::stream::{Stream, TryStreamExt as _};
use futures::Future;

use crate::cpufreq::{self, Cache};
use crate::Result;

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Values {
    id: u64,
}

impl Values {
    pub fn available() -> impl Future<Output = Result<bool>> {
        cpufreq::available()
    }

    pub fn exists(id: u64) -> impl Future<Output = Result<bool>> {
        cpufreq::exists(id)
    }

    pub fn ids() -> impl Stream<Item = Result<u64>> {
        cpufreq::ids()
    }

    pub fn all() -> impl Stream<Item = Result<Self>> {
        cpufreq::ids().map_ok(Self::new)
    }

    pub fn new(id: u64) -> Self {
        Self { id }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn cpuinfo_max_freq(&self) -> impl Future<Output=Result<u64>> {
        cpufreq::cpuinfo_max_freq(self.id)
    }

    pub fn cpuinfo_min_freq(&self) -> impl Future<Output=Result<u64>> {
        cpufreq::cpuinfo_min_freq(self.id)
    }

    pub fn scaling_cur_freq(&self) -> impl Future<Output=Result<u64>> {
        cpufreq::scaling_cur_freq(self.id)
    }

    pub fn scaling_driver(&self) -> impl Future<Output=Result<String>> {
        cpufreq::scaling_driver(self.id)
    }

    pub fn scaling_governor(&self) -> impl Future<Output=Result<String>> {
        cpufreq::scaling_governor(self.id)
    }

    pub fn scaling_available_governors(&self) -> impl Future<Output=Result<Vec<String>>> {
        cpufreq::scaling_available_governors(self.id)
    }

    pub fn scaling_max_freq(&self) -> impl Future<Output=Result<u64>> {
        cpufreq::scaling_max_freq(self.id)
    }

    pub fn scaling_min_freq(&self) -> impl Future<Output=Result<u64>> {
        cpufreq::scaling_min_freq(self.id)
    }

    pub async fn set_scaling_governor(&self, v: impl AsRef<str>) -> Result<()> {
        cpufreq::set_scaling_governor(self.id, v.as_ref()).await
    }

    pub fn set_scaling_max_freq(&self, v: u64) -> impl Future<Output=Result<()>> {
        cpufreq::set_scaling_max_freq(self.id, v)
    }

    pub fn set_scaling_min_freq(&self, v: u64) -> impl Future<Output=Result<()>> {
        cpufreq::set_scaling_min_freq(self.id, v)
    }
}

impl From<Cache> for Values {
    fn from(v: Cache) -> Self {
        Self::new(v.id())
    }
}

impl From<&Cache> for Values {
    fn from(v: &Cache) -> Self {
        Self::new(v.id())
    }
}
