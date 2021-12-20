pub mod cpu;
pub mod cpufreq;
pub mod drm;
pub mod i915;
pub mod nv;
pub mod pstate;
pub mod rapl;
mod util;

use std::{path::PathBuf, fmt::Display};

pub use nvml_wrapper::error::NvmlError;
pub use tokio::io::Error as IoError;
pub use crate::cpu::Cpu;
pub use crate::cpufreq::Policy as CpufreqPolicy;
pub use crate::drm::Card as DrmCard;
pub use crate::i915::Card as I915Card;
pub use crate::nv::Card as NvCard;
pub use crate::pstate::policy::Policy as PstatePolicy;
pub use crate::pstate::system::System as PstateSystem;
pub use crate::rapl::constraint::Constraint as RaplConstraint;
pub use crate::rapl::zone::Zone as RaplZone;

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
    #[error("Error: {0}")]
    NonSequitor(String),

    #[error("sysfs {op}: {path}: {source}")]
    SysfsIo {
        #[source]
        source: IoError,
        path: PathBuf,
        op: Op,
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
        #[source]
        source: NvmlError,
        device: String,
        method: &'static str,
        op: Op,
    },
}

impl Error {
    fn non_sequitor(s: impl Display) -> Self {
        let s = s.to_string();
        Self::NonSequitor(s)
    }

    fn sysfs_read(source: IoError, path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let op = Op::Read;
        Self::SysfsIo { source, path, op }
    }

    fn sysfs_write(source: IoError, path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let op = Op::Write;
        Self::SysfsIo { source, path, op }
    }

    fn sysfs_parse(path: impl Into<PathBuf>, ty: &'static str, value: impl Display) -> Self {
        let path = path.into();
        let value = value.to_string();
        Self::SysfsParse { path, ty, value }
    }

    fn nvml_init(error: &'static NvmlError) -> Self {
        Self::NvmlInit(error)
    }

    fn nvml_read(source: NvmlError, device: impl Display, method: &'static str) -> Self {
        let device = device.to_string();
        let op = Op::Read;
        Self::NvmlIo {
            source,
            device,
            method,
            op,
        }
    }

    fn nvml_write(source: NvmlError, device: impl Display, method: &'static str) -> Self {
        let device = device.to_string();
        let op = Op::Write;
        Self::NvmlIo {
            source,
            device,
            method,
            op,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, Eq, Ord, Hash, PartialEq, PartialOrd)]
pub struct BusId {
    pub bus: String,
    pub id: String,
}

