#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "nvml")]
pub mod nvml;
pub mod sysfs;

use async_trait::async_trait;

#[async_trait]
pub trait Feature {
    async fn present() -> bool;
}

#[async_trait]
pub trait Policy {
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

#[cfg(feature = "nvml")]
use nvml::Nvml;
use sysfs::{Cpu, Cpufreq, IntelPstate, IntelRapl, I915};

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
impl Policy for Machine {
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
        if let Some(r) = &self.cpu {
            r.write().await;
        }
        if let Some(r) = &self.cpufreq {
            r.write().await;
        }
        if let Some(r) = &self.i915 {
            r.write().await;
        }
        if let Some(r) = &self.intel_pstate {
            r.write().await;
        }
        if let Some(r) = &self.intel_rapl {
            r.write().await;
        }
        #[cfg(feature = "nvml")]
        if let Some(r) = &self.nvml {
            r.write().await;
        }
    }
}
