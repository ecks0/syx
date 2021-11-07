use std::collections::HashMap;
use std::fmt::Display;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tokio::io::ErrorKind as IoErrorKind;

use crate::path::profile as path;
use crate::Chain;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{path}: {message}")]
    De { path: PathBuf, message: String },

    #[error("{path}: {message}")]
    Io { path: PathBuf, message: String },

    #[error("No profile config exists in {search_paths:#?}")]
    ConfigMissing { search_paths: Vec<String> },

    #[error("Profile '{profile}' not found in {path}")]
    ProfileMissing { path: PathBuf, profile: String },

    #[error("Corrupt state file at {path}")]
    StateCorrupt { path: PathBuf },

    #[error("Previous profile state not found at {path}")]
    StateMissing { path: PathBuf },

    #[error("{activity}: unable to determine xdg user state directory")]
    StatePathMissing { activity: String },

    #[error("{message}")]
    Se { message: String },
}

impl Error {
    fn de<S: Display>(path: PathBuf, message: S) -> Self {
        let message = message.to_string();
        Self::De { path, message }
    }

    fn io<S: Display>(path: PathBuf, message: S) -> Self {
        let message = message.to_string();
        Self::Io { path, message }
    }

    fn config_missing(search_paths: Vec<PathBuf>) -> Self {
        let search_paths = search_paths
            .into_iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect();
        Self::ConfigMissing { search_paths }
    }

    fn profile_missing<S: Display>(path: PathBuf, profile: S) -> Self {
        let profile = profile.to_string();
        Self::ProfileMissing { path, profile }
    }

    fn state_corrupt(path: PathBuf) -> Self { Self::StateCorrupt { path } }

    fn state_missing(path: PathBuf) -> Self { Self::StateMissing { path } }

    fn state_path_missing<S: Display>(activity: S) -> Self {
        let action = activity.to_string();
        Self::StatePathMissing { activity: action }
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
    async fn recent() -> Result<Self> {
        use tokio::fs::read_to_string;
        let p = if let Some(p) = path::state_path().await {
            p
        } else {
            return Err(Error::state_path_missing("Read recent profile"));
        };
        let s = match read_to_string(&p).await {
            Ok(s) => s,
            Err(e) => match e.kind() {
                crate::IoErrorKind::NotFound => return Err(Error::state_missing(p)),
                _ => return Err(Error::io(p, e)),
            },
        };
        match serde_yaml::from_str(&s) {
            Ok(r) => Ok(r),
            Err(_) => Err(Error::state_corrupt(p)),
        }
    }

    pub(crate) async fn new<S: Into<String>>(name: S) -> Result<Self> {
        let name = name.into();
        if name == Self::RECENT_PROFILE_STR {
            Self::recent().await
        } else {
            match path::config_path().await {
                Some(path) => Ok(Self { name, path }),
                None => Err(Error::config_missing(Self::paths().await)),
            }
        }
    }

    pub(crate) async fn read(&self) -> Result<Chain> {
        log::debug!("Reading profiles from {}", self.path.display());
        let p = self.path.clone();
        match tokio::fs::read_to_string(&p).await {
            Ok(s) => match serde_yaml::from_str::<HashMap<String, Chain>>(&s) {
                Ok(cf) => match cf.into_iter().find(|(n, _)| n == &self.name) {
                    Some((_, c)) => Ok(c),
                    None => Err(Error::profile_missing(p, &self.name)),
                },
                Err(e) => Err(Error::de(p, e)),
            },
            Err(e) => match e.kind() {
                IoErrorKind::NotFound => Err(Error::config_missing(Self::paths().await)),
                _ => Err(Error::io(p, e)),
            },
        }
    }

    pub(crate) async fn set_recent(&self) -> Result<()> {
        use tokio::fs::{create_dir_all, write};
        let p = if let Some(p) = path::state_path().await {
            p
        } else {
            return Err(Error::state_path_missing("Write recent profile"));
        };
        if let Some(parent) = p.parent() {
            if !parent.is_dir() {
                create_dir_all(parent)
                    .await
                    .map_err(|e| Error::io(parent.to_path_buf(), e))?;
            }
        }
        let s = serde_yaml::to_string(self).map_err(Error::se)?;
        write(&p, s.as_bytes()).await.map_err(|e| Error::io(p, e))?;
        Ok(())
    }
}
