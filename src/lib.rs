// #[cfg(feature = "cli")]
// pub mod cli;
pub mod cpu;
pub mod cpufreq;
pub mod drm;
pub mod i915;
pub mod intel_pstate;
pub mod intel_rapl;
#[cfg(feature = "nvml")]
pub mod nv;
pub mod prelude;
pub(crate) mod sysfs;
pub(crate) mod util;

use async_trait::async_trait;

#[async_trait]
pub trait Read {
    async fn read(&mut self);
}

#[async_trait]
pub trait Write {
    async fn write(&self);
}

pub trait Values {
    fn is_empty(&self) -> bool;

    fn clear(&mut self);
}

#[async_trait]
pub trait Single: Default + Read + Send + Sized + Values {
    async fn load() -> Self {
        let mut s = Self::default();
        s.read().await;
        s
    }
}

#[async_trait]
pub trait Multi: Default + PartialEq + Read + Send + Sized + Values {
    type Id: Send + Sized;

    async fn ids() -> Vec<Self::Id>;

    async fn load(id: Self::Id) -> Self {
        let mut s = Self::new(id);
        s.read().await;
        s
    }

    async fn load_all() -> Vec<Self> {
        let mut all = vec![];
        for id in Self::ids().await {
            let s = Self::load(id).await;
            all.push(s);
        }
        all
    }

    fn new(id: Self::Id) -> Self {
        let mut s = Self::default();
        s.set_id(id);
        s
    }

    fn id(&self) -> Self::Id;

    fn set_id(&mut self, v: Self::Id);
}

#[async_trait]
pub trait Feature {
    async fn present() -> bool;
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct System {
    pub cpu: cpu::System,
    pub cpufreq: cpufreq::System,
    pub i915: i915::System,
    pub intel_pstate: intel_pstate::System,
    pub intel_rapl: intel_rapl::System,
    #[cfg(feature = "nvml")]
    pub nv: nv::System,
}

#[async_trait]
impl Read for System {
    async fn read(&mut self) {
        self.cpu = cpu::System::load().await;
        self.cpufreq = cpufreq::System::load().await;
        self.i915 = i915::System::load().await;
        self.intel_pstate = intel_pstate::System::load().await;
        self.intel_rapl = intel_rapl::System::load().await;
        #[cfg(feature = "nvml")] {
            self.nv = nv::System::load().await;
        }
    }
}

#[async_trait]
impl Write for System {
    async fn write(&self) {
        if !self.cpufreq.is_empty() || !self.intel_pstate.is_empty() {
            let mut ids = vec![];
            ids.extend(self.cpufreq.devices.iter().map(|d| d.id));
            ids.extend(self.intel_pstate.devices.iter().map(|d| d.id));
            ids.sort_unstable();
            ids.dedup();
            let ids = util::set_cpus_online(ids).await;
            self.cpufreq.write().await;
            self.intel_pstate.write().await;
            util::wait_for_cpu_related().await;
            util::set_cpus_offline(ids).await;
        }
        self.cpu.write().await;
        self.intel_rapl.write().await;
        self.i915.write().await;
        #[cfg(feature = "nvml")]
        self.nv.write().await;
    }
}

impl Values for System {
    fn is_empty(&self) -> bool {
        self.eq(&Self::default())
    }

    fn clear(&mut self) {
        *self = Self::default();
    }
}

#[async_trait]
impl Single for System {}
