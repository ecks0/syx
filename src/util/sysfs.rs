use std::fmt::{Debug, Display};
use std::path::{Path, PathBuf};
use std::result::Result as StdResult;

use tokio::io::{AsyncReadExt as _, Error as IoError};

use crate::{Error, Result};

pub(crate) async fn read_bool(path: &Path) -> Result<bool> {
    let val = read_string(path).await?;
    match val.as_str() {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(Error::sysfs_parse(path, "bool", val)),
    }
}

pub(crate) async fn write_bool(path: &Path, val: bool) -> Result<()> {
    let val = match val {
        false => "0",
        true => "1",
    };
    handle_write(path, tokio::fs::write(path, val).await, val)
}

pub(crate) async fn read_ids(path: &Path, prefix: &str) -> Result<Vec<u64>> {
    async fn read_ids(path: &Path, prefix: &str) -> StdResult<Vec<u64>, IoError> {
        let mut ids = vec![];
        let mut dir = tokio::fs::read_dir(path).await?;
        while let Some(ent) = dir.next_entry().await? {
            if let Some(file_name) = ent.path().file_name() {
                if let Some(file_name) = file_name.to_str() {
                    if let Some(value) = file_name.strip_prefix(prefix) {
                        if let Ok(value) = value.parse::<u64>() {
                            ids.push(value);
                            ids.sort_unstable();
                        }
                    }
                }
            }
        }
        Ok(ids)
    }
    handle_read(path, read_ids(path, prefix).await)
}

pub(crate) async fn read_indices(path: &Path) -> Result<Vec<u64>> {
    let s = read_string(path).await?;
    let mut ids = vec![];
    for item in s.split(',') {
        let parts: Vec<&str> = item.split('-').collect();
        match &parts[..] {
            [id] => ids.push(
                id.parse::<u64>()
                    .map_err(|_| Error::sysfs_parse(path, "indices: index", item))?,
            ),
            [start, end] => {
                let start = start
                    .parse::<u64>()
                    .map_err(|_| Error::sysfs_parse(path, "indices: start", item))?;
                let end = 1 + end
                    .parse::<u64>()
                    .map_err(|_| Error::sysfs_parse(path, "indices: end", item))?;
                ids.extend(start..end);
            },
            _ => return Err(Error::sysfs_parse(path, "indices", item)),
        }
    }
    ids.sort_unstable();
    ids.dedup();
    Ok(ids)
}

pub(crate) async fn read_link(path: &Path) -> Result<PathBuf> {
    handle_read(path, tokio::fs::read_link(path).await)
}

pub(crate) async fn read_link_name(path: &Path) -> Result<String> {
    let val = read_link(path)
        .await?
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();
    Ok(val)
}

pub(crate) async fn read_string(path: &Path) -> Result<String> {
    async fn read_string(path: &Path) -> StdResult<String, IoError> {
        let mut f = tokio::fs::File::open(path).await?;
        let mut s = String::with_capacity(256);
        f.read_to_string(&mut s).await?;
        let s = s.trim_end_matches('\n').to_string();
        Ok(s)
    }
    handle_read(path, read_string(path).await).map(|s| s.trim_end_matches('\n').to_string())
}

pub(crate) async fn write_string(path: &Path, val: &str) -> Result<()> {
    handle_write(path, tokio::fs::write(path, val).await, val)
}

pub(crate) async fn read_string_list(path: &Path, delim: char) -> Result<Vec<String>> {
    read_string(path)
        .await
        .map(|s| s.split(delim).map(String::from).collect())
}

pub(crate) async fn read_u64(path: &Path) -> Result<u64> {
    let val = read_string(path).await?;
    val.parse::<u64>()
        .map_err(|_| Error::sysfs_parse(path, "u64", val))
}

pub(crate) async fn write_u64(path: &Path, val: u64) -> Result<()> {
    write_string(path, &val.to_string()).await
}

fn handle_read<T: Debug>(path: &Path, result: StdResult<T, IoError>) -> Result<T> {
    #[cfg(feature = "logging")]
    match &result {
        Ok(v) => {
            log::debug!("OK sysfs r {} {:?}", path.display(), v);
        },
        Err(e) => {
            if let Some(errno) = e.raw_os_error() {
                log::warn!(
                    "ERR sysfs r {:?} {}",
                    nix::errno::Errno::from_i32(errno),
                    path.display()
                );
            } else {
                log::error!("ERR sysfs r {} {}", path.display(), e);
            }
        },
    }
    result.map_err(|e| Error::sysfs_read(e, path))
}

fn handle_write<T, S: Display>(path: &Path, result: StdResult<T, IoError>, _value: S) -> Result<T> {
    #[cfg(feature = "logging")]
    match &result {
        Ok(_) => {
            log::debug!("OK sysfs w {} {}", path.display(), _value);
        },
        Err(e) => {
            if let Some(errno) = e.raw_os_error() {
                log::error!(
                    "ERR sysfs w {:?} {} {}",
                    nix::errno::Errno::from_i32(errno),
                    path.display(),
                    _value,
                );
            } else {
                log::error!("ERR sysfs w {} {} {}", path.display(), _value, e);
            }
        },
    }
    result.map_err(|e| Error::sysfs_write(e, path))
}
