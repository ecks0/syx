use measurements::Power;
use zysfs::types::{self as sysfs, tokio::Read as _};
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, atomic::{AtomicBool, Ordering}},
    time::{Duration, Instant},
};
use tokio::{sync::Mutex, time::sleep};

// Samples rapl energy usage at a regular interval.
#[derive(Clone, Debug)]
pub struct RaplSampler {
    zone: sysfs::intel_rapl::ZoneId,
    interval: Duration,
    values: Arc<Mutex<VecDeque<u64>>>,
    working: Arc<AtomicBool>,
}

impl RaplSampler {

    const COUNT: usize = 11;

    pub async fn all(interval: Duration) -> Option<Vec<RaplSampler>> {
        sysfs::intel_rapl::Policy::ids().await
            .map(|zones| zones
                .into_iter()
                .map(|zone| Self::new(zone, interval))
                .collect())
    }

    pub fn new(zone: sysfs::intel_rapl::ZoneId, interval: Duration) -> Self {
        Self {
            zone,
            interval,
            values: Default::default(),
            working: Default::default(),
        }
    }

    pub fn working(&self) -> bool { self.working.load(Ordering::Acquire) }

    fn swap_working(&mut self, v: bool) -> bool { self.working.swap(v, Ordering::Acquire) }

    async fn poll(&self) -> Option<u64> {
        sysfs::intel_rapl::io::tokio::energy_uj(self.zone.zone, self.zone.subzone).await.ok()
    }

    async fn work(&mut self) {
        let mut begin = Instant::now();
        while self.working() {
            match self.poll().await {
                Some(v) => {
                    let mut values = self.values.lock().await;
                    values.push_back(v);
                    while values.len() > Self::COUNT { values.pop_front(); }
                    drop(values);
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

    pub fn zone(&self) -> sysfs::intel_rapl::ZoneId { self.zone }

    pub async fn values(&self) -> Vec<u64> {  { self.values.lock().await.clone() }.into() }

    pub async fn watt_seconds(&self) -> Option<Power> {
        let samples = self.values().await;
        let uw = match samples.len() {
            v if v < 2 => return None,
            v if v < Self::COUNT/2 =>
                (1..samples.len())
                    .map(|i| samples[i] - samples[i - 1])
                    .max(),
            _ => {
                let mut deltas: Vec<u64> =
                    (1..samples.len())
                        .map(|i| samples[i] - samples[i - 1])
                        .collect();
                deltas.sort_unstable();
                Some(deltas[deltas.len()/2])
            },
        };
        uw.map(|uw|
            Power::from_microwatts(
                (uw as f64 * 10f64.powf(6.) / self.interval.as_micros() as f64).round()
            ))
    }
}

// Manage a collection of `RaplSampler`s.
#[derive(Clone, Debug)]
pub struct RaplSamplers {
    samplers: HashMap<sysfs::intel_rapl::ZoneId, RaplSampler>,
}

impl RaplSamplers {

    pub async fn working(&self) -> bool { self.samplers.values().any(|s| s.working()) }

    pub async fn start(&mut self) { for s in self.samplers.values_mut() { s.start().await; } }

    pub async fn stop(&mut self) { for s in self.samplers.values_mut() { s.stop().await; } }

    pub async fn watt_seconds(&self, zone: sysfs::intel_rapl::ZoneId) -> Option<Power> {
        self.samplers.get(&zone)?.watt_seconds().await
    }
}

impl From<Vec<RaplSampler>> for RaplSamplers {
    fn from(v: Vec<RaplSampler>) -> Self {
        let samplers = v
            .into_iter()
            .map(|c| (c.zone(), c))
            .collect();
        Self {
            samplers,
        }
    }
}
