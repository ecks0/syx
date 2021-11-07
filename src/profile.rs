use std::collections::HashMap;
use std::fmt::Display;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::path::profile as path;
use crate::Chain;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{path}: {message}")]
    De { path: String, message: String },

    #[error("{path}: {message}")]
    Io { path: String, message: String },

    #[error("No profile config exists in {search_paths:#?}")]
    NoConfig { search_paths: Vec<String> },

    #[error("Profile '{profile}' not found in {path}")]
    NoProfile { path: String, profile: String },

    #[error("Previous profile state not found at {path}")]
    NoState { path: String },

    #[error("{action}: unable to determine xdg user state directory")]
    NoStatePath { action: String },

    #[error("{message}")]
    Se { message: String },
}

impl Error {
    fn path_to_str(p: &Path) -> String { p.to_string_lossy().into_owned() }

    fn de<S: Display>(path: &Path, message: S) -> Self {
        let path = Self::path_to_str(path);
        let message = message.to_string();
        Self::De { path, message }
    }

    fn io<S: Display>(path: &Path, message: S) -> Self {
        let path = Self::path_to_str(path);
        let message = message.to_string();
        Self::Io { path, message }
    }

    fn no_config(search_paths: Vec<PathBuf>) -> Self {
        let search_paths = search_paths.into_iter().map(|p| Self::path_to_str(&p)).collect();
        Self::NoConfig { search_paths }
    }

    fn no_profile<S: Display>(path: &Path, profile: S) -> Self {
        let path = Self::path_to_str(path);
        let profile = profile.to_string();
        Self::NoProfile { path, profile }
    }

    fn no_state(path: &Path) -> Self {
        let path = Self::path_to_str(path);
        Self::NoState { path }
    }

    fn no_state_path<S: Display>(action: S) -> Self {
        let action = action.to_string();
        Self::NoStatePath { action }
    }

    fn se<S: Display>(message: S) -> Self {
        let message = message.to_string();
        Self::Se { message }
    }
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Profile {
    name: String,
    path: PathBuf,
}

impl Profile {
    // String to be interpreted as an alias for the name of the most recently
    // applied profile.
    const RECENT_PROFILE_STR: &'static str = "_";

    pub(crate) async fn paths() -> Vec<PathBuf> { path::config_paths().await }

    // Return the most recently applied profile.
    async fn recent() -> Result<Option<Self>> {
        use tokio::fs::{read_to_string, remove_file};
        let p = if let Some(p) = path::state_path().await {
            p
        } else {
            return Err(Error::no_state_path("Read recent profile"));
        };
        let s = match read_to_string(&p).await {
            Ok(s) => s,
            Err(e) => match e.kind() {
                crate::IoErrorKind::NotFound => return Err(Error::no_state(&p)),
                _ => return Err(Error::io(&p, e)),
            },
        };
        match serde_yaml::from_str(&s) {
            Ok(r) => Ok(Some(r)),
            Err(e) => {
                log::error!(
                    "ERR knobs r Profile::previous() Discarding recent profile state due to parse \
                     error:"
                );
                log::error!("ERR knobs r Profile::previous() {}: {}", p.display(), e);
                remove_file(&p).await.map_err(|e| Error::io(&p, e))?;
                Ok(None)
            },
        }
    }

    pub(crate) async fn new<S: Into<String>>(name: S) -> Result<Option<Self>> {
        let name = name.into();
        if name == Self::RECENT_PROFILE_STR {
            Self::recent().await
        } else {
            let s = path::config_path().await.map(|path| Self { name, path });
            Ok(s)
        }
    }

    pub(crate) async fn read(&self) -> Result<Chain> {
        let path = if let Some(p) = path::config_path().await {
            p
        } else {
            return Err(Error::no_config(Self::paths().await));
        };
        log::debug!("Reading profiles from {}", path.display());
        match tokio::fs::read_to_string(&path).await {
            Ok(s) => match serde_yaml::from_str::<HashMap<String, Chain>>(&s) {
                Ok(p) => match p.into_iter().find(|(n, _)| n == &self.name) {
                    Some((_, c)) => Ok(c),
                    None => Err(Error::no_profile(&path, &self.name)),
                },
                Err(e) => Err(Error::de(&path, e)),
            },
            Err(e) => Err(Error::io(&path, e)),
        }
    }

    pub(crate) async fn set_recent(&self) -> Result<()> {
        use tokio::fs::{create_dir_all, write};
        let p = if let Some(p) = path::state_path().await {
            p
        } else {
            return Err(Error::no_state_path("Write recent profile"));
        };
        if let Some(parent) = p.parent() {
            if !parent.is_dir() {
                create_dir_all(parent).await.map_err(|e| Error::io(parent, e))?;
            }
        }
        let s = serde_yaml::to_string(self).map_err(Error::se)?;
        write(&p, s.as_bytes()).await.map_err(|e| Error::io(&p, e))?;
        Ok(())
    }
}
