use futures::stream::{Stream, TryStreamExt as _};
use futures::Future;

use crate::nvml::{self, Cache};
use crate::Result;

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Values {
    id: u64,
}

impl Values {
    pub fn available() -> impl Future<Output = Result<bool>> {
        nvml::available()
    }

    pub fn exists(id: u64) -> impl Future<Output = Result<bool>> {
        nvml::exists(id)
    }

    pub fn ids() -> impl Stream<Item = Result<u64>> {
        nvml::ids()
    }

    pub fn all() -> impl Stream<Item = Result<Self>> {
        nvml::ids().map_ok(Self::new)
    }

    pub fn new(id: u64) -> Self {
        Self { id }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn gfx_freq(&self) -> impl Future<Output = Result<u32>> {
        nvml::gfx_freq(self.id)
    }

    pub fn gfx_max_freq(&self) -> impl Future<Output = Result<u32>> {
        nvml::gfx_max_freq(self.id)
    }

    pub fn mem_freq(&self) -> impl Future<Output = Result<u32>> {
        nvml::mem_freq(self.id)
    }

    pub fn mem_max_freq(&self) -> impl Future<Output = Result<u32>> {
        nvml::mem_max_freq(self.id)
    }

    pub fn sm_freq(&self) -> impl Future<Output = Result<u32>> {
        nvml::sm_freq(self.id)
    }

    pub fn video_freq(&self) -> impl Future<Output = Result<u32>> {
        nvml::video_freq(self.id)
    }

    pub fn video_max_freq(&self) -> impl Future<Output = Result<u32>> {
        nvml::video_max_freq(self.id)
    }

    pub fn mem_total(&self) -> impl Future<Output = Result<u64>> {
        nvml::mem_total(self.id)
    }

    pub fn mem_used(&self) -> impl Future<Output = Result<u64>> {
        nvml::mem_used(self.id)
    }

    pub fn name(&self) -> impl Future<Output = Result<String>> {
        nvml::name(self.id)
    }

    pub fn power(&self) -> impl Future<Output = Result<u32>> {
        nvml::power(self.id)
    }

    pub fn power_limit(&self) -> impl Future<Output = Result<u32>> {
        nvml::power_limit(self.id)
    }

    pub fn power_max_limit(&self) -> impl Future<Output = Result<u32>> {
        nvml::power_max_limit(self.id)
    }

    pub fn power_min_limit(&self) -> impl Future<Output = Result<u32>> {
        nvml::power_min_limit(self.id)
    }

    pub fn set_gfx_freq(&self, min: u32, max: u32) -> impl Future<Output = Result<()>> {
        nvml::set_gfx_freq(self.id, min, max)
    }

    pub fn reset_gfx_freq(&self) -> impl Future<Output = Result<()>> {
        nvml::reset_gfx_freq(self.id)
    }

    pub fn set_power_limit(&self, v: u32) -> impl Future<Output = Result<()>> {
        nvml::set_power_limit(self.id, v)
    }

    pub fn reset_power_limit(&self) -> impl Future<Output = Result<()>> {
        nvml::reset_power_limit(self.id)
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
