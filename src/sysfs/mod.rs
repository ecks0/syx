mod log;

use std::path::{Path, PathBuf};

use tokio::io::Error as IoError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] IoError),

    #[error("{path}: Invalid value for {ty}: {value}")]
    Parse {
        path: PathBuf,
        ty: &'static str,
        value: String,
    },
}

impl Error {
    fn parse<P, S>(path: P, ty: &'static str, value: S) -> Self
    where
        P: Into<PathBuf>,
        S: std::fmt::Display,
    {
        let path = path.into();
        let value = value.to_string();
        Self::Parse { path, ty, value }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) async fn read_bool(path: &Path) -> Result<bool> {
    let val = read_str(path).await?;
    match val.as_str() {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(Error::parse(path, "bool", val)),
    }
}

pub(crate) async fn write_bool(path: &Path, val: bool) -> Result<()> {
    let val = match val {
        false => "0",
        true => "1",
    };
    log::write(path, tokio::fs::write(path, val).await, val)
}

pub(crate) async fn read_ids(path: &Path, prefix: &str) -> Result<Vec<u64>> {
    async fn read_ids(path: &Path, prefix: &str) -> std::result::Result<Vec<u64>, IoError> {
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
    log::read(path, read_ids(path, prefix).await)
}

pub(crate) async fn read_indices(path: &Path) -> Result<Vec<u64>> {
    let s = read_str(path).await?;
    let mut ids = vec![];
    for item in s.split(',') {
        let parts: Vec<&str> = item.split('-').collect();
        match &parts[..] {
            [id] => ids.push(
                id.parse::<u64>()
                    .map_err(|_| Error::parse(path, "indices: index", item))?,
            ),
            [start, end] => {
                let start = start
                    .parse::<u64>()
                    .map_err(|_| Error::parse(path, "indices: start", item))?;
                let end = 1 + end
                    .parse::<u64>()
                    .map_err(|_| Error::parse(path, "indices: end", item))?;
                ids.extend(start..end);
            },
            _ => return Err(Error::parse(path, "indices", item)),
        }
    }
    ids.sort_unstable();
    ids.dedup();
    Ok(ids)
}

pub(crate) async fn read_link(path: &Path) -> Result<PathBuf> {
    log::read(path, tokio::fs::read_link(path).await)
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

pub(crate) async fn read_to_trimmed_string(path: &Path) -> std::result::Result<String, IoError> {
    tokio::fs::read_to_string(path)
        .await
        .map(|s| s.trim_end_matches('\n').to_string())
}

pub(crate) async fn read_str(path: &Path) -> Result<String> {
    log::read(path, read_to_trimmed_string(path).await)
        .map(|s| s.trim_end_matches('\n').to_string())
}

pub(crate) async fn write_str(path: &Path, val: &str) -> Result<()> {
    log::write(path, tokio::fs::write(path, val).await, val)
}

pub(crate) async fn read_str_list(path: &Path, delim: char) -> Result<Vec<String>> {
    read_str(path)
        .await
        .map(|s| s.split(delim).map(String::from).collect())
}

pub(crate) async fn read_u64(path: &Path) -> Result<u64> {
    let val = read_str(path).await?;
    val.parse::<u64>()
        .map_err(|_| Error::parse(path, "u64", val))
}

pub(crate) async fn write_u64(path: &Path, val: u64) -> Result<()> {
    write_str(path, &val.to_string()).await
}
