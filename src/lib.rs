pub mod cpu;
pub mod cpufreq;
pub mod drm;
pub mod i915;
pub mod nv;
pub mod prelude;
pub mod pstate;
pub mod rapl;
pub(crate) mod util;

use std::path::PathBuf;

use async_trait::async_trait;
pub use nvml_wrapper::error::NvmlError;
pub use tokio::io::Error as IoError;

pub use crate::cpu::System as Cpu;
pub use crate::cpufreq::System as Cpufreq;
pub use crate::drm::System as Drm;
pub use crate::i915::System as I915;
pub use crate::nv::System as Nv;
pub use crate::pstate::System as Pstate;
pub use crate::rapl::System as Rapl;

#[derive(Clone, Debug)]
pub enum Op {
    Read,
    Write,
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read => write!(f, "read"),
            Self::Write => write!(f, "write"),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("sysfs {op}: {path}: {source}")]
    SysfsIo {
        path: PathBuf,
        op: Op,
        #[source]
        source: IoError,
    },

    #[error("sysfs parse: {path}: Invalid value for {ty}: {value:?}")]
    SysfsParse {
        path: PathBuf,
        ty: &'static str,
        value: String,
    },

    #[error("nvml init: {0}")]
    NvmlInit(&'static NvmlError),

    #[error("nvml list devices: {0}")]
    NvmlListDevices(#[source] NvmlError),

    #[error("nvml {op} id {device}: {source}")]
    NvmlIo {
        device: u64,
        op: Op,
        method: &'static str,
        #[source]
        source: NvmlError,
    },
}

impl Error {
    fn sysfs_read(path: impl Into<PathBuf>, source: IoError) -> Self {
        let path = path.into();
        let op = Op::Read;
        Self::SysfsIo { path, op, source }
    }

    fn sysfs_write(path: impl Into<PathBuf>, source: IoError) -> Self {
        let path = path.into();
        let op = Op::Write;
        Self::SysfsIo { path, op, source }
    }

    fn sysfs_parse(path: impl Into<PathBuf>, ty: &'static str, value: impl Into<String>) -> Self {
        let path = path.into();
        let value = value.into();
        Self::SysfsParse { path, ty, value }
    }

    fn nvml_init(error: &'static NvmlError) -> Self {
        Self::NvmlInit(error)
    }

    fn nvml_read(device: u64, method: &'static str, source: NvmlError) -> Self {
        let op = Op::Read;
        Self::NvmlIo {
            device,
            op,
            method,
            source,
        }
    }

    fn nvml_write(device: u64, method: &'static str, source: NvmlError) -> Self {
        let op = Op::Write;
        Self::NvmlIo {
            device,
            op,
            method,
            source,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

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

    fn clear(&mut self) -> &mut Self;
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

    fn set_id(&mut self, v: Self::Id) -> &mut Self;
}

#[async_trait]
pub trait Feature {
    async fn present() -> bool;
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct System {
    cpu: cpu::System,
    cpufreq: cpufreq::System,
    i915: i915::System,
    intel_pstate: pstate::System,
    intel_rapl: rapl::System,
    nv: nv::System,
}

impl System {
    // TODO
}

#[async_trait]
impl Read for System {
    async fn read(&mut self) {
        self.cpu = cpu::System::load().await;
        self.cpufreq = cpufreq::System::load().await;
        self.i915 = i915::System::load().await;
        self.intel_pstate = pstate::System::load().await;
        self.intel_rapl = rapl::System::load().await;
        self.nv = nv::System::load().await;
    }
}

#[async_trait]
impl Write for System {
    async fn write(&self) {
        if !(self.cpufreq.is_empty() && self.intel_pstate.is_empty()) {
            let mut ids = self
                .cpufreq
                .devices()
                .map(|d| d.id())
                .chain(self.intel_pstate.devices().map(|d| d.id()))
                .collect::<Vec<_>>();
            ids.sort_unstable();
            ids.dedup();
            let ids = util::cpu::set_online(ids).await;
            self.cpufreq.write().await;
            self.intel_pstate.write().await;
            util::cpu::wait_for_write().await;
            util::cpu::set_offline(ids).await;
        }
        self.cpu.write().await;
        self.intel_rapl.write().await;
        self.i915.write().await;
        self.nv.write().await;
    }
}

impl Values for System {
    fn is_empty(&self) -> bool {
        self.eq(&Self::default())
    }

    fn clear(&mut self) -> &mut Self {
        *self = Self::default();
        self
    }
}

#[async_trait]
impl Single for System {}
