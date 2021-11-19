use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;
use tokio::time::sleep;

use crate::sysfs::intel_rapl::{energy_uj, Device, ZoneId};
use crate::Resource as _;

fn umean(n: &[u64]) -> f64 {
    let sum: u64 = n.iter().sum();
    let sum = sum as f64;
    let len = n.len() as f64;
    sum / len
}

fn umedian(n: &[u64]) -> f64 {
    let len = n.len();
    let mid = len / 2;
    if len % 2 == 0 {
        umean(&n[(mid - 1)..(mid + 1)])
    } else {
        n[mid] as f64
    }
}

// Sample rapl energy usage at a regular interval.
#[derive(Clone, Debug)]
pub struct Sampler {
    zone: ZoneId,
    interval: Duration,
    values: Arc<Mutex<VecDeque<u64>>>,
    working: Arc<AtomicBool>,
}

impl Sampler {
    const COUNT: usize = 11;

    pub async fn all(interval: Duration) -> Samplers {
        Device::ids()
            .await
            .into_iter()
            .map(|zone| Self::new(zone, interval))
            .collect::<Vec<_>>()
            .into()
    }

    pub fn new(zone: ZoneId, interval: Duration) -> Self {
        Self {
            zone,
            interval,
            values: Default::default(),
            working: Default::default(),
        }
    }

    pub fn working(&self) -> bool {
        self.working.load(Ordering::Acquire)
    }

    fn swap_working(&mut self, v: bool) -> bool {
        self.working.swap(v, Ordering::Acquire)
    }

    async fn poll(&self) -> Option<u64> {
        energy_uj(self.zone.zone, self.zone.subzone).await.ok()
    }

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

    pub async fn start(&mut self) {
        if self.swap_working(true) {
            return;
        }
        let mut worker = self.clone();
        tokio::task::spawn(async move {
            worker.work().await;
        });
    }

    pub async fn stop(&mut self) {
        self.swap_working(false);
    }

    async fn values(&self) -> Vec<u64> {
        { self.values.lock().await.clone() }.into()
    }

    pub async fn watt_seconds(&self) -> Option<f64> {
        let samples = self.values().await;
        if samples.len() < 2 {
            return None;
        }
        let mut deltas: Vec<u64> = (1..samples.len())
            .map(|i| samples[i] - samples[i - 1])
            .collect();
        deltas.sort_unstable();
        let uw = umedian(&deltas);
        let w = uw / 10f64.powf(6.);
        let ivl_us = self.interval.as_micros() as f64;
        let sec_us = 10f64.powf(6.);
        let w_per_sec = w * sec_us / ivl_us;
        Some(w_per_sec)
    }
}

// Manage a collection of `RaplSampler`s.
#[derive(Clone, Debug)]
pub struct Samplers {
    samplers: HashMap<ZoneId, Sampler>,
}

impl Samplers {
    pub async fn working(&self) -> bool {
        self.samplers.values().any(|s| s.working())
    }

    pub fn count(&self) -> usize {
        self.samplers.len()
    }

    pub async fn start(&mut self) {
        for s in self.samplers.values_mut() {
            s.start().await;
        }
    }

    pub async fn stop(&mut self) {
        for s in self.samplers.values_mut() {
            s.stop().await;
        }
    }

    pub async fn watt_seconds(&self, zone: ZoneId) -> Option<f64> {
        self.samplers.get(&zone)?.watt_seconds().await
    }
}

impl From<Vec<Sampler>> for Samplers {
    fn from(v: Vec<Sampler>) -> Self {
        let samplers = v.into_iter().map(|c| (c.zone, c)).collect();
        Self { samplers }
    }
}
