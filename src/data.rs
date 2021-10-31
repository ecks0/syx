use zysfs::types as sysfs;
use zysfs::types::tokio::Read as _;
use zysfs::io::intel_rapl::tokio::energy_uj;
use std::{collections::{HashMap, VecDeque}, sync::{Arc, atomic::{AtomicBool, Ordering}}, time::{Duration, Instant}};
use tokio::{
    sync::{Mutex, OnceCell},
    time::sleep,
};
use measurements::Power;

// Simple runtime counter. Initialized on the first get.
#[derive(Clone, Debug)]
pub(crate) struct RuntimeCounter;

impl RuntimeCounter {

    pub async fn get() -> Duration {
        static START: OnceCell<Instant> = OnceCell::const_new();
        async fn make() -> Instant { Instant::now() }
        let then = *START.get_or_init(make).await;
        Instant::now() - then
    }

    pub async fn initialize() { Self::get().await; }
}

// Cache resource ids to eliminate unnecessary sysfs io. Automatic lazy initialization.
#[derive(Clone, Debug)]
pub(crate) struct ResourceIdCache;

impl ResourceIdCache {

    pub async fn cpu() -> Option<Vec<u64>> {
        static CPU_IDS_CACHED: OnceCell<Option<Vec<u64>>> = OnceCell::const_new();
        async fn ids() -> Option<Vec<u64>> { sysfs::cpu::Policy::ids().await }
        CPU_IDS_CACHED.get_or_init(ids).await.clone()
    }

    pub async fn drm() -> Option<Vec<u64>> {
        static DRM_IDS_CACHED: OnceCell<Option<Vec<u64>>> = OnceCell::const_new();
        async fn ids() -> Option<Vec<u64>> { sysfs::drm::Card::ids().await }
        DRM_IDS_CACHED.get_or_init(ids).await.clone()
    }

    pub async fn drm_i915() -> Option<Vec<u64>> {
        use sysfs::drm::io::tokio::driver;
        static DRM_I915_IDS_CACHED: OnceCell<Option<Vec<u64>>> = OnceCell::const_new();
        async fn ids() -> Option<Vec<u64>> {
            let mut ids = vec![];
            if let Some(drm_ids) = ResourceIdCache::drm().await {
                for id in drm_ids {
                    if let Ok("i915") = driver(id).await.as_deref() {
                        ids.push(id);
                    }
                }
            }
            if ids.is_empty() { None } else { Some(ids) }
        }
        DRM_I915_IDS_CACHED.get_or_init(ids).await.clone()
    }

    #[cfg(feature = "nvml")]
    pub async fn nvml() -> Option<Vec<u64>> {
        static NVML_IDS_CACHED: OnceCell<Option<Vec<u64>>> = OnceCell::const_new();
        async fn ids() -> Option<Vec<u64>> {
            nvml_facade::Nvml::ids()
                .map(|ids| ids.into_iter().map(u64::from).collect())
        }
        NVML_IDS_CACHED.get_or_init(ids).await.clone()
    }
}

// Sample rapl energy usage at a regular interval.
#[derive(Clone, Debug)]
pub struct RaplEnergySampler {
    zone: sysfs::intel_rapl::ZoneId,
    interval: Duration,
    count: usize,
    values: Arc<Mutex<VecDeque<u64>>>,
    working: Arc<AtomicBool>,
}

impl RaplEnergySampler {

    pub async fn all(interval: Duration, count: usize) -> Option<Vec<RaplEnergySampler>> {
        sysfs::intel_rapl::Policy::ids().await
            .map(|zones| zones
                .into_iter()
                .map(|zone| Self::new(zone, interval, count))
                .collect::<Vec<RaplEnergySampler>>())
            .and_then(|s| if s.is_empty() { None } else { Some(s) })
    }

    pub fn new(zone: sysfs::intel_rapl::ZoneId, interval: Duration, count: usize) -> Self {
        Self {
            zone,
            interval,
            count,
            values: Default::default(),
            working: Default::default(),
        }
    }

    pub fn working(&self) -> bool { self.working.load(Ordering::Acquire) }

    fn swap_working(&mut self, v: bool) -> bool { self.working.swap(v, Ordering::Acquire) }

    async fn poll(&self) -> Option<u64> { energy_uj(self.zone.zone, self.zone.subzone).await.ok() }

    async fn work(&mut self) {
        let mut begin = Instant::now();
        while self.working() {
            match self.poll().await {
                Some(v) => {
                    let mut guard = self.values.lock().await;
                    guard.push_back(v);
                    while guard.len() > self.count { guard.pop_front(); }
                    drop(guard);
                },
                None => {
                    self.swap_working(false);
                    break;
                },
            }
            let s = self.interval - (Instant::now() - begin).min(self.interval);
            if !s.is_zero() { sleep(s).await }
            begin = Instant::now();
        }
    }

    pub async fn start(&mut self) {
        if self.swap_working(true) { return; }
        let mut this = self.clone();
        tokio::task::spawn(async move { this.work().await; });
    }

    pub async fn stop(&mut self) { self.swap_working(false); }

    pub async fn clear(&mut self) { self.values.lock().await.clear(); }

    pub fn zone(&self) -> sysfs::intel_rapl::ZoneId { self.zone }

    pub async fn values(&self) -> Vec<u64> { self.values.lock().await.clone().into() }

    pub async fn watt_seconds_max(&self) -> Option<Power> {
        let samples = self.values().await;
        if samples.len() < 2 { return None; }
        (1..samples.len())
            .map(|i| samples[i] - samples[i - 1])
            .max()
            .map(|uw|
                Power::from_microwatts(
                    uw as f64 * 10f64.powf(6.) / self.interval.as_micros() as f64
                ))
    }
}

// Manage a collection of `RaplEnergySampler`s.
#[derive(Clone, Debug)]
pub struct RaplEnergySamplers {
    samplers: HashMap<sysfs::intel_rapl::ZoneId, RaplEnergySampler>,
}

impl RaplEnergySamplers {

    pub async fn working(&self) -> bool { self.samplers.values().any(|s| s.working()) }

    pub async fn start(&mut self) { for s in self.samplers.values_mut() { s.start().await; } }

    pub async fn stop(&mut self) { for s in self.samplers.values_mut() { s.stop().await; } }

    pub async fn clear(&mut self) { for s in self.samplers.values_mut() { s.clear().await; } }

    pub async fn watt_seconds_max(&self, zone: sysfs::intel_rapl::ZoneId) -> Option<Power> {
        self.samplers.get(&zone)?.watt_seconds_max().await
    }
}

impl From<Vec<RaplEnergySampler>> for RaplEnergySamplers {
    fn from(v: Vec<RaplEnergySampler>) -> Self {
        let samplers = v
            .into_iter()
            .map(|c| (c.zone(), c))
            .collect();
        Self {
            samplers,
        }
    }
}

// Static instance of for `RaplEnergySamplers`. Must be initialized.
#[derive(Clone, Debug)]
pub struct RaplEnergySamplersInstance;

static RAPL_ENERGY_SAMPLERS: OnceCell<Option<RaplEnergySamplers>> = OnceCell::const_new();

impl RaplEnergySamplersInstance {

    const INTERVAL: Duration = Duration::from_millis(100);
    const COUNT: usize = 11;

    async fn get_or_init() -> Option<RaplEnergySamplers> {
        async fn samplers() -> Option<RaplEnergySamplers> {
            RaplEnergySampler::all(
                RaplEnergySamplersInstance::INTERVAL,
                RaplEnergySamplersInstance::COUNT
            ).await.map(|c| c.into())
        }
        RAPL_ENERGY_SAMPLERS.get_or_init(samplers).await.clone()
    }

    pub fn initialized() -> bool { RAPL_ENERGY_SAMPLERS.initialized() }

    pub async fn initialize() {
        if let Some(mut s) = Self::get_or_init().await { s.clear().await; }
    }

    pub async fn get() -> Option<RaplEnergySamplers> {
        if Self::initialized() { Self::get_or_init().await } else { None }
    }
}
