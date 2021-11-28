#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "nvml")]
pub mod nvml;
pub mod sysfs;

use std::time::Duration;

use async_trait::async_trait;
#[cfg(feature = "nvml")]
use nvml::Nvml;
use sysfs::{Cpu, Cpufreq, IntelPstate, IntelRapl, I915};
use tokio::time::sleep;

#[async_trait]
pub trait Feature {
    async fn present() -> bool;
}

#[async_trait]
pub trait Values {
    type Id: Sized + Send;
    type Output: Sized + Send;

    async fn ids() -> Vec<Self::Id>;

    async fn all() -> Vec<Self::Output> {
        let mut all = vec![];
        for id in Self::ids().await {
            if let Some(output) = Self::read(id).await {
                all.push(output);
            }
        }
        all
    }

    async fn read(id: Self::Id) -> Option<Self::Output>;

    async fn write(&self);
}

async fn wait_for_cpu_onoff() {
    const WAIT_FOR_CPU_ONOFF: Duration = Duration::from_millis(300);
    sleep(WAIT_FOR_CPU_ONOFF).await
}

async fn wait_for_cpu_related() {
    const WAIT_FOR_CPU_RELATED: Duration = Duration::from_millis(100);
    sleep(WAIT_FOR_CPU_RELATED).await
}

async fn set_cpus_online(cpu_ids: Vec<u64>) -> Vec<u64> {
    if cpu_ids.is_empty() {
        return Default::default();
    }
    let offline = sysfs::cpu::devices_offline().await.unwrap_or_default();
    let mut onlined = vec![];
    for cpu_id in cpu_ids {
        if offline.contains(&cpu_id) && sysfs::cpu::set_online(cpu_id, true).await.is_ok() {
            onlined.push(cpu_id);
        }
    }
    if !onlined.is_empty() {
        wait_for_cpu_onoff().await;
    }
    onlined
}

async fn set_cpus_offline(cpu_ids: Vec<u64>) -> Vec<u64> {
    if cpu_ids.is_empty() {
        return Default::default();
    }
    let online = sysfs::cpu::devices_online().await.unwrap_or_default();
    let mut offlined = vec![];
    for cpu_id in cpu_ids {
        if online.contains(&cpu_id) && sysfs::cpu::set_online(cpu_id, false).await.is_ok() {
            offlined.push(cpu_id);
        }
    }
    if !offlined.is_empty() {
        wait_for_cpu_onoff().await;
    }
    offlined
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Machine {
    pub cpu: Option<Cpu>,
    pub cpufreq: Option<Cpufreq>,
    pub i915: Option<I915>,
    pub intel_pstate: Option<IntelPstate>,
    pub intel_rapl: Option<IntelRapl>,
    #[cfg(feature = "nvml")]
    pub nvml: Option<Nvml>,
}

#[async_trait]
impl Values for Machine {
    type Id = ();
    type Output = Self;

    async fn ids() -> Vec<()> {
        vec![()]
    }

    async fn read(_: ()) -> Option<Self> {
        let cpu = Cpu::read(()).await;
        let cpufreq = Cpufreq::read(()).await;
        let i915 = I915::read(()).await;
        let intel_pstate = IntelPstate::read(()).await;
        let intel_rapl = IntelRapl::read(()).await;
        #[cfg(feature = "nvml")]
        let nvml = Nvml::read(()).await;
        let s = Self {
            cpu,
            cpufreq,
            i915,
            intel_pstate,
            intel_rapl,
            #[cfg(feature = "nvml")]
            nvml,
        };
        Some(s)
    }

    async fn write(&self) {
        if self.cpufreq.is_some() || self.intel_pstate.is_some() {
            let mut onlined = vec![];
            if let Some(r) = &self.cpufreq {
                onlined.extend(r.devices.iter().map(|d| d.id));
            }
            if let Some(r) = &self.intel_pstate {
                onlined.extend(r.devices.iter().map(|d| d.id));
            }
            onlined.sort_unstable();
            onlined.dedup();
            let onlined = set_cpus_online(onlined).await;
            if let Some(r) = self.cpufreq.as_ref() {
                r.write().await;
            }
            if let Some(r) = self.cpufreq.as_ref() {
                r.write().await;
            }
            wait_for_cpu_related().await;
            set_cpus_offline(onlined).await;
        }
        if let Some(r) = &self.cpu {
            r.write().await;
        }
        if let Some(r) = &self.intel_rapl {
            r.write().await;
        }
        if let Some(r) = &self.i915 {
            r.write().await;
        }
        #[cfg(feature = "nvml")]
        if let Some(r) = &self.nvml {
            r.write().await;
        }
    }
}

#[derive(Clone, Debug)]
struct Value<T> {
    cell: Option<T>,
}
