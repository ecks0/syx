mod cell;
pub mod cpu;
pub mod cpufreq;
pub mod drm;
pub mod i915;
pub mod nv;
pub mod pstate;
pub mod rapl;
mod sysfs;

use std::path::PathBuf;

pub use nvml_wrapper::error::NvmlError;
pub use tokio::io::Error as IoError;

pub(crate) use crate::cell::Cached;
pub use crate::cpu::Cpu;
pub use crate::cpufreq::Cpu as CpufreqCpu;
pub use crate::drm::Card as DrmCard;
pub use crate::i915::Card as I915Card;
pub use crate::nv::Card as NvCard;
pub use crate::pstate::{Cpu as PstateCpu, System as PstateSystem};
pub use crate::rapl::{Constraint as RaplConstraint, Zone as RaplZone};

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
