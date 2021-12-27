use futures::Future;

use crate::nvml;
use crate::util::stream::prelude::*;
use crate::util::cell::Cached;
use crate::Result;

#[derive(Clone, Debug)]
pub struct Cache {
    id: u64,
    gfx_freq: Cached<u32>,
    gfx_max_freq: Cached<u32>,
    mem_freq: Cached<u32>,
    mem_max_freq: Cached<u32>,
    sm_freq: Cached<u32>,
    sm_max_freq: Cached<u32>,
    video_freq: Cached<u32>,
    video_max_freq: Cached<u32>,
    mem_total: Cached<u64>,
    mem_used: Cached<u64>,
    name: Cached<String>,
    power: Cached<u32>,
    power_limit: Cached<u32>,
    power_limit_max: Cached<u32>,
    power_limit_min: Cached<u32>,
}

impl Cache {
    pub fn available() -> impl Future<Output=Result<bool>> {
        nvml::available()
    }

    pub fn exists(id: u64) -> impl Future<Output=Result<bool>> {
        nvml::exists(id)
    }

    pub fn ids() -> impl Stream<Item=Result<u64>> {
        nvml::ids()
    }

    pub fn new(id: u64) -> Self {
        Self {
            id,
            gfx_freq: Cached::default(),
            gfx_max_freq: Cached::default(),
            mem_freq: Cached::default(),
            mem_max_freq: Cached::default(),
            sm_freq: Cached::default(),
            sm_max_freq: Cached::default(),
            video_freq: Cached::default(),
            video_max_freq: Cached::default(),
            mem_total: Cached::default(),
            mem_used: Cached::default(),
            name: Cached::default(),
            power: Cached::default(),
            power_limit: Cached::default(),
            power_limit_max: Cached::default(),
            power_limit_min: Cached::default(),
        }
    }

    pub async fn clear(&self) {
        tokio::join!(
            self.gfx_freq.clear(),
            self.gfx_max_freq.clear(),
            self.mem_freq.clear(),
            self.mem_max_freq.clear(),
            self.sm_freq.clear(),
            self.sm_max_freq.clear(),
            self.video_freq.clear(),
            self.video_max_freq.clear(),
            self.mem_total.clear(),
            self.mem_used.clear(),
            self.name.clear(),
            self.power.clear(),
            self.power_limit.clear(),
            self.power_limit_max.clear(),
            self.power_limit_min.clear(),
        );
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub async fn gfx_freq(&self) -> Result<u32> {
        self.gfx_freq.get_or_load(nvml::gfx_freq(self.id)).await
    }

    pub async fn gfx_max_freq(&self) -> Result<u32> {
        self.gfx_max_freq.get_or_load(nvml::gfx_max_freq(self.id)).await
    }

    pub async fn mem_freq(&self) -> Result<u32> {
        self.mem_freq.get_or_load(nvml::mem_freq(self.id)).await
    }

    pub async fn mem_max_freq(&self) -> Result<u32> {
        self.mem_max_freq.get_or_load(nvml::mem_max_freq(self.id)).await
    }

    pub async fn sm_freq(&self) -> Result<u32> {
        self.sm_freq.get_or_load(nvml::sm_freq(self.id)).await
    }

    pub async fn video_freq(&self) -> Result<u32> {
        self.video_freq.get_or_load(nvml::video_freq(self.id)).await
    }

    pub async fn video_max_freq(&self) -> Result<u32> {
        self.video_max_freq
            .get_or_load(nvml::video_max_freq(self.id))
            .await
    }

    pub async fn mem_total(&self) -> Result<u64> {
        self.mem_total.get_or_load(nvml::mem_total(self.id)).await
    }

    pub async fn mem_used(&self) -> Result<u64> {
        self.mem_used.get_or_load(nvml::mem_used(self.id)).await
    }

    pub async fn name(&self) -> Result<String> {
        self.name.get_or_load(nvml::name(self.id)).await
    }

    pub async fn power(&self) -> Result<u32> {
        self.power.get_or_load(nvml::power(self.id)).await
    }

    pub async fn power_limit(&self) -> Result<u32> {
        self.power_limit.get_or_load(nvml::power_limit(self.id)).await
    }

    pub async fn power_max_limit(&self) -> Result<u32> {
        self.power_limit_max
            .get_or_load(nvml::power_max_limit(self.id))
            .await
    }

    pub async fn power_min_limit(&self) -> Result<u32> {
        self.power_limit_min
            .get_or_load(nvml::power_min_limit(self.id))
            .await
    }

    pub async fn set_gfx_freq(&self, min: u32, max: u32) -> Result<()> {
        self.gfx_freq
            .clear_if_ok(nvml::set_gfx_freq(self.id, min, max))
            .await
    }

    pub async fn reset_gfx_freq(&self) -> Result<()> {
        self.gfx_freq.clear_if_ok(nvml::reset_gfx_freq(self.id)).await
    }

    pub async fn set_power_limit(&self, v: u32) -> Result<()> {
        self.power_limit
            .clear_if_ok(nvml::set_power_limit(self.id, v))
            .await
    }

    pub async fn reset_power_limit(&self) -> Result<()> {
        self.power_limit
            .clear_if_ok(nvml::reset_power_limit(self.id))
            .await
    }
}
