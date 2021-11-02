use zysfs::types::{self as sysfs, tokio::Read as _};
use std::time::{Duration, Instant};
use tokio::sync::OnceCell;

pub async fn cpu_ids() -> Option<Vec<u64>> {
    static CPU_IDS: OnceCell<Option<Vec<u64>>> = OnceCell::const_new();
    async fn cpu_ids() -> Option<Vec<u64>> { sysfs::cpu::Policy::ids().await }
    CPU_IDS.get_or_init(cpu_ids).await.clone()
}

pub async fn drm_ids() -> Option<Vec<u64>> {
    static DRM_IDS: OnceCell<Option<Vec<u64>>> = OnceCell::const_new();
    async fn drm_ids() -> Option<Vec<u64>> { sysfs::drm::Card::ids().await }
    DRM_IDS.get_or_init(drm_ids).await.clone()
}

pub async fn drm_i915_ids() -> Option<Vec<u64>> {
    use sysfs::drm::io::tokio::driver;
    static DRM_I915_IDS: OnceCell<Option<Vec<u64>>> = OnceCell::const_new();
    async fn drm_i915_ids() -> Option<Vec<u64>> {
        let mut ids = vec![];
        if let Some(drm_ids) = drm_ids().await {
            for id in drm_ids {
                if let Ok("i915") = driver(id).await.as_deref() {
                    ids.push(id);
                }
            }
        }
        if ids.is_empty() { None } else { Some(ids) }
    }
    DRM_I915_IDS.get_or_init(drm_i915_ids).await.clone()
}

#[cfg(feature = "nvml")]
pub async fn nvml_ids() -> Option<Vec<u64>> {
    static NVML_IDS: OnceCell<Option<Vec<u64>>> = OnceCell::const_new();
    async fn nvml_ids() -> Option<Vec<u64>> {
        nvml_facade::Nvml::ids()
            .map(|ids| ids.into_iter().map(u64::from).collect())
    }
    NVML_IDS.get_or_init(nvml_ids).await.clone()
}

#[derive(Debug)]
pub struct Counter;

impl Counter {
    pub async fn get() -> Instant {
        static COUNTER: OnceCell<Instant> = OnceCell::const_new();
        async fn start() -> Instant { Instant::now() }
        *COUNTER.get_or_init(start).await
    }

    pub async fn delta() -> Duration {
        let then = Self::get().await;
        Instant::now() - then
    }
}
