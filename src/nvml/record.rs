use futures::Future;

use crate::nvml;
use crate::util::stream::prelude::*;
use crate::Result;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Record {
    id: u64,
    gfx_freq: Option<u32>,
    gfx_max_freq: Option<u32>,
    mem_freq: Option<u32>,
    mem_max_freq: Option<u32>,
    sm_freq: Option<u32>,
    sm_max_freq: Option<u32>,
    video_freq: Option<u32>,
    video_max_freq: Option<u32>,
    mem_total: Option<u64>,
    mem_used: Option<u64>,
    name: Option<String>,
    power: Option<u32>,
    power_limit: Option<u32>,
    power_max_limit: Option<u32>,
    power_min_limit: Option<u32>,
}

impl Record {
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
            gfx_freq: None,
            gfx_max_freq: None,
            mem_freq: None,
            mem_max_freq: None,
            sm_freq: None,
            sm_max_freq: None,
            video_freq: None,
            video_max_freq: None,
            mem_total: None,
            mem_used: None,
            name: None,
            power: None,
            power_limit: None,
            power_max_limit: None,
            power_min_limit: None,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn is_empty(&self) -> bool {
        self == &Self::new(self.id)
    }

    pub async fn read(&mut self) -> bool {
        self.gfx_freq = nvml::gfx_freq(self.id).await.ok();
        self.gfx_max_freq = nvml::gfx_max_freq(self.id).await.ok();
        self.mem_freq = nvml::mem_freq(self.id).await.ok();
        self.mem_max_freq = nvml::mem_max_freq(self.id).await.ok();
        self.sm_freq = nvml::sm_freq(self.id).await.ok();
        self.sm_max_freq = nvml::sm_max_freq(self.id).await.ok();
        self.video_freq = nvml::video_freq(self.id).await.ok();
        self.video_max_freq = nvml::video_max_freq(self.id).await.ok();
        self.mem_total = nvml::mem_total(self.id).await.ok();
        self.mem_used = nvml::mem_used(self.id).await.ok();
        self.name = nvml::name(self.id).await.ok();
        self.power = nvml::power(self.id).await.ok();
        self.power_limit = nvml::power_limit(self.id).await.ok();
        self.power_max_limit = nvml::power_max_limit(self.id).await.ok();
        self.power_min_limit = nvml::power_min_limit(self.id).await.ok();
        !self.is_empty()
    }

    pub async fn gfx_freq(&self) -> Option<u32> {
        self.gfx_freq
    }

    pub async fn gfx_max_freq(&self) -> Option<u32> {
        self.gfx_max_freq
    }

    pub async fn mem_freq(&self) -> Option<u32> {
        self.mem_freq
    }

    pub async fn mem_max_freq(&self) -> Option<u32> {
        self.mem_max_freq
    }

    pub async fn sm_freq(&self) -> Option<u32> {
        self.sm_freq
    }

    pub async fn video_freq(&self) -> Option<u32> {
        self.video_freq
    }

    pub async fn video_max_freq(&self) -> Option<u32> {
        self.video_max_freq
    }

    pub async fn mem_total(&self) -> Option<u64> {
        self.mem_total
    }

    pub async fn mem_used(&self) -> Option<u64> {
        self.mem_used
    }

    pub async fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub async fn power(&self) -> Option<u32> {
        self.power
    }

    pub async fn power_limit(&self) -> Option<u32> {
        self.power_limit
    }

    pub async fn power_max_limit(&self) -> Option<u32> {
        self.power_max_limit
    }

    pub async fn power_min_limit(&self) -> Option<u32> {
        self.power_min_limit
    }
}
