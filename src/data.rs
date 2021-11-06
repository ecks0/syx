use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use measurements::Power;
use tokio::sync::Mutex;
use tokio::time::sleep;
use zysfs::io::intel_rapl::tokio::energy_uj;
use zysfs::types as sysfs;
use zysfs::types::tokio::Read as _;

fn mean(n: Vec<u64>) -> f64 {
    let sum: u64 = n.iter().sum();
    sum as f64 / n.len() as f64
}

// fn median(mut n: Vec<u64>) -> u64 {
//     n.sort_unstable();
//     let mid = n.len() / 2;
//     if n.len() % 2 == 0 {
//         mean(n[mid - 1..mid].into()).round() as u64
//     } else {
//         n[mid]
//     }
// }

// Sample rapl energy usage at a regular interval.
#[derive(Clone, Debug)]
pub(crate) struct RaplSampler {
    zone: sysfs::intel_rapl::ZoneId,
    interval: Duration,
    values: Arc<Mutex<VecDeque<u64>>>,
    working: Arc<AtomicBool>,
}

impl RaplSampler {
    const COUNT: usize = 11;

    pub(crate) async fn all(interval: Duration) -> Option<Vec<RaplSampler>> {
        sysfs::intel_rapl::Policy::ids()
            .await
            .map(|zones| zones.into_iter().map(|zone| Self::new(zone, interval)).collect())
    }

    pub(crate) fn new(zone: sysfs::intel_rapl::ZoneId, interval: Duration) -> Self {
        Self {
            zone,
            interval,
            values: Default::default(),
            working: Default::default(),
        }
    }

    pub(crate) fn working(&self) -> bool { self.working.load(Ordering::Acquire) }

    fn swap_working(&mut self, v: bool) -> bool { self.working.swap(v, Ordering::Acquire) }

    async fn poll(&self) -> Option<u64> { energy_uj(self.zone.zone, self.zone.subzone).await.ok() }

    async fn work(&mut self) {
        let mut begin = Instant::now();
        while self.working() {
            match self.poll().await {
                Some(v) => {
                    let mut values = self.values.lock().await;
                    values.push_back(v);
                    while values.len() > Self::COUNT {
                        values.pop_front();
                    }
                    drop(values);
                },
                None => {
                    self.swap_working(false);
                    break;
                },
            }
            let s = self.interval - (Instant::now() - begin).min(self.interval);
            sleep(s).await;
            begin = Instant::now();
        }
    }

    pub(crate) async fn start(&mut self) {
        if self.swap_working(true) {
            return;
        }
        let mut worker = self.clone();
        tokio::task::spawn(async move {
            worker.work().await;
        });
    }

    pub(crate) async fn stop(&mut self) { self.swap_working(false); }

    pub(crate) fn zone(&self) -> sysfs::intel_rapl::ZoneId { self.zone }

    async fn values(&self) -> Vec<u64> { { self.values.lock().await.clone() }.into() }

    pub(crate) async fn watt_seconds(&self) -> Option<Power> {
        let samples = self.values().await;
        if samples.len() < 2 {
            return None;
        }
        let deltas = (1..samples.len()).map(|i| samples[i] - samples[i - 1]);
        if samples.len() < Self::COUNT / 2 - 1 {
            deltas.max()
        } else {
            Some(mean(deltas.collect()).round() as u64)
        }
        .map(|uw| {
            Power::from_microwatts(
                (uw as f64 * f64::powf(10., 6.) / self.interval.as_micros() as f64).round(),
            )
        })
    }
}

// Manage a collection of `RaplSampler`s.
#[derive(Clone, Debug)]
pub(crate) struct RaplSamplers {
    samplers: HashMap<sysfs::intel_rapl::ZoneId, RaplSampler>,
}

impl RaplSamplers {
    pub(crate) async fn working(&self) -> bool { self.samplers.values().any(|s| s.working()) }

    pub(crate) async fn start(&mut self) {
        for s in self.samplers.values_mut() {
            s.start().await;
        }
    }

    pub(crate) async fn stop(&mut self) {
        for s in self.samplers.values_mut() {
            s.stop().await;
        }
    }

    pub(crate) async fn watt_seconds(&self, zone: sysfs::intel_rapl::ZoneId) -> Option<Power> {
        self.samplers.get(&zone)?.watt_seconds().await
    }
}

impl From<Vec<RaplSampler>> for RaplSamplers {
    fn from(v: Vec<RaplSampler>) -> Self {
        let samplers = v.into_iter().map(|c| (c.zone(), c)).collect();
        Self { samplers }
    }
}
