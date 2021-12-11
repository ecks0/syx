use std::fmt::{Debug, Display};
use std::path::Path;

use log::{debug, error, log_enabled, warn, Level};
use nix::errno::Errno;
use tokio::io::Error as IoError;

use crate::util::sysfs::Error;

pub fn read<T: Debug>(path: &Path, result: Result<T, IoError>) -> Result<T, Error> {
    match &result {
        Ok(v) => {
            if log_enabled!(Level::Debug) {
                debug!("OK sysfs r {} {:?}", path.display(), v);
            }
        },
        Err(e) => {
            if let Some(errno) = e.raw_os_error() {
                if log_enabled!(Level::Warn) {
                    warn!(
                        "ERR sysfs r {:?} {}",
                        Errno::from_i32(errno),
                        path.display()
                    );
                }
            } else if log_enabled!(Level::Error) {
                error!("ERR sysfs r {} {}", path.display(), e);
            }
        },
    }
    result.map_err(|e| e.into())
}

pub fn write<T, S: Display>(path: &Path, result: Result<T, IoError>, value: S) -> Result<T, Error> {
    match &result {
        Ok(_) => {
            if log_enabled!(Level::Debug) {
                debug!("OK sysfs w {} {}", path.display(), value);
            }
        },
        Err(e) => {
            if log_enabled!(Level::Error) {
                if let Some(errno) = e.raw_os_error() {
                    error!(
                        "ERR sysfs w {:?} {} {}",
                        Errno::from_i32(errno),
                        path.display(),
                        value
                    );
                } else {
                    error!("ERR sysfs w {} {} {}", path.display(), value, e);
                }
            }
        },
    }
    result.map_err(|e| e.into())
}
