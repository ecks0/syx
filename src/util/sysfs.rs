use std::fmt::{Debug, Display};
use std::path::{Path, PathBuf};
use std::result::Result as StdResult;

use async_stream::try_stream;
use futures::stream::Stream;
use tokio::fs::DirEntry;
use tokio::io::Error as IoError;

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

fn read_dir_ents(path: &Path) -> impl Stream<Item = Result<DirEntry>> {
    let path = path.to_path_buf();
    try_stream! {
        let f = tokio::fs::read_dir(&path);
        let mut dir = handle_read(&path, f.await)?;
        while let Some(ent) = dir.next_entry()
            .await
            .map_err(|e| Error::sysfs_read(e, &path))?
        {
            yield ent;
        }
    }
}

pub(crate) fn read_ids(path: &Path, prefix: &str) -> impl Stream<Item = Result<u64>> {
    let path = path.to_path_buf();
    let prefix = prefix.to_string();
    try_stream! {
        for await ent in read_dir_ents(&path) {
            let ent = ent?;
            let v = ent
                .path()
                .file_name()
                .and_then(|v| v.to_str())
                .and_then(|v| v.strip_prefix(&prefix))
                .and_then(|v| v.parse::<u64>().ok());
            if let Some(v) = v {
                yield v;
            }
        }
    }
}

#[rustfmt::skip]
pub(crate) fn read_indices(path: &Path) -> impl Stream<Item=Result<u64>> {
    let path = path.to_path_buf();
    try_stream! {
        let s = read_string(&path).await?;
        let i = if s.is_empty() {
            0..0
        } else {
            let p: Vec<_> = s.split('-').collect();
            match &p[..] {
                [id] => {
                    let id = id.parse::<u64>()
                        .map_err(|_| Error::sysfs_parse(&path, "indices: index", &s))?;
                    id..(id+1)
                },
                [start, end] => {
                    let start = start
                        .parse::<u64>()
                        .map_err(|_| Error::sysfs_parse(&path, "indices: start", &s))?;
                    let end = end
                        .parse::<u64>()
                        .map_err(|_| Error::sysfs_parse(&path, "indices: end", s))?;
                    start..(end+1)
                },
                _ => Err(Error::sysfs_parse(&path, "indices", s))?,
            }
        };
        for v in i {
            yield v;
        }
    }
}

pub(crate) async fn read_link(path: &Path) -> Result<PathBuf> {
    handle_read(path, tokio::fs::read_link(path).await)
}

pub(crate) async fn read_link_name(path: &Path) -> Result<String> {
    let val = read_link(path)
        .await?
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("") // FIXME
        .to_string();
    Ok(val)
}

pub(crate) async fn read_string(path: &Path) -> Result<String> {
    handle_read(path, tokio::fs::read_to_string(path).await)
        .map(|s| s.trim_end_matches('\n').to_string())
}

pub(crate) async fn write_string(path: &Path, val: &str) -> Result<()> {
    handle_write(path, tokio::fs::write(path, val).await, val)
}

pub(crate) async fn read_string_list(path: &Path, delim: char) -> Result<Vec<String>> {
    read_string(path).await.map(|s| {
        s.trim_end_matches(delim)
            .split(delim)
            .map(String::from)
            .collect()
    })
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
            log::trace!("OK sysfs r {} {:?}", path.display(), v);
        },
        Err(e) => {
            if let Some(errno) = e.raw_os_error() {
                log::trace!(
                    "ERR sysfs r {:?} {}",
                    nix::errno::Errno::from_i32(errno),
                    path.display()
                );
            } else {
                log::trace!("ERR sysfs r {} {}", path.display(), e);
            }
        },
    }
    result.map_err(|e| Error::sysfs_read(e, path))
}

fn handle_write<T, S: Display>(path: &Path, result: StdResult<T, IoError>, _value: S) -> Result<T> {
    #[cfg(feature = "logging")]
    match &result {
        Ok(_) => {
            log::trace!("OK sysfs w {} {}", path.display(), _value);
        },
        Err(e) => {
            if let Some(errno) = e.raw_os_error() {
                log::trace!(
                    "ERR sysfs w {:?} {} {}",
                    nix::errno::Errno::from_i32(errno),
                    path.display(),
                    _value,
                );
            } else {
                log::trace!("ERR sysfs w {} {} {}", path.display(), _value, e);
            }
        },
    }
    result.map_err(|e| Error::sysfs_write(e, path))
}
