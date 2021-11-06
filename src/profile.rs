use std::collections::HashMap;
use std::path::{Path, PathBuf};

use nix::unistd::Uid;
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;

use crate::{env, path, Chain};

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

    #[error("{message}")]
    Se { message: String },
}

impl Error {
    fn path_to_str(p: &Path) -> String { p.to_string_lossy().into_owned() }

    fn de(path: &Path, message: String) -> Self {
        let path = Self::path_to_str(path);
        Self::De { path, message }
    }

    fn io(path: &Path, message: String) -> Self {
        let path = Self::path_to_str(path);
        Self::Io { path, message }
    }

    fn no_config(search_paths: Vec<PathBuf>) -> Self {
        let search_paths = search_paths.into_iter().map(|p| Self::path_to_str(&p)).collect();
        Self::NoConfig { search_paths }
    }

    fn no_profile(path: &Path, profile: String) -> Self {
        let path = Self::path_to_str(path);
        Self::NoProfile { path, profile }
    }

    fn no_state(path: &Path) -> Self {
        let path = Self::path_to_str(path);
        Self::NoState { path }
    }

    fn se(message: String) -> Self { Self::Se { message } }
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
    const RECENT_PROFILE_ALIAS: &'static str = "_";

    pub(crate) async fn paths() -> Vec<PathBuf> { config_paths().await }

    // Return the most recently applied profile.
    async fn recent() -> Result<Option<Self>> {
        use tokio::fs::{read_to_string, remove_file};
        let p = if let Some(p) = state_path().await {
            p
        } else {
            return Ok(None);
        };
        if !p.is_file() {
            return Err(Error::no_state(&p));
        };
        let s = read_to_string(&p).await.map_err(|e| Error::io(&p, e.to_string()))?;
        match serde_yaml::from_str(&s) {
            Ok(r) => Ok(Some(r)),
            Err(e) => {
                log::error!(
                    "ERR knobs r Profile::previous() Discarding recent profile state due to parse \
                     error:"
                );
                log::error!(
                    "ERR knobs r Profile::previous() {}: {}",
                    p.display(),
                    e.to_string()
                );
                remove_file(&p).await.map_err(|e| Error::io(&p, e.to_string()))?;
                Ok(None)
            },
        }
    }

    pub(crate) async fn new<S: Into<String>>(name: S) -> Result<Option<Self>> {
        let name = name.into();
        if name == Self::RECENT_PROFILE_ALIAS {
            Self::recent().await
        } else {
            let s = config_path().await.map(|path| Self { name, path });
            Ok(s)
        }
    }

    pub(crate) async fn read(&self) -> Result<Chain> {
        let path = if let Some(p) = config_path().await {
            p
        } else {
            return Err(Error::no_config(config_paths().await));
        };
        log::debug!("Reading profiles from {}", path.display());
        match tokio::fs::read_to_string(&path).await {
            Ok(s) => match serde_yaml::from_str::<HashMap<String, Chain>>(&s) {
                Ok(p) => match p.into_iter().find(|(n, _)| n == &self.name) {
                    Some((_, c)) => Ok(c),
                    None => Err(Error::no_profile(&path, self.name.clone())),
                },
                Err(e) => Err(Error::de(&path, e.to_string())),
            },
            Err(e) => Err(Error::io(&path, e.to_string())),
        }
    }

    pub(crate) async fn set_recent(&self) -> Result<()> {
        use tokio::fs::{create_dir_all, write};
        let p = if let Some(p) = state_path().await {
            p
        } else {
            return Ok(());
        };
        if let Some(parent) = p.parent() {
            if !parent.is_dir() {
                create_dir_all(parent).await.map_err(|e| Error::io(parent, e.to_string()))?;
            }
        }
        let s = serde_yaml::to_string(self).map_err(|e| Error::se(e.to_string()))?;
        write(&p, s.as_bytes()).await.map_err(|e| Error::io(&p, e.to_string()))?;
        Ok(())
    }
}

// e.g. ~/.config/knobs/{CONFIG_DIR_NAME} or /etc/knobs/{CONFIG_DIR_NAME}
const CONFIG_DIR_NAME: &str = "profile";
// Suffix of the environment variable.
const CONFIG_ENV_SUFFIX: &str = "PROFILE_CONFIG";
// The `default` part of `default.yaml`.
const DEFAULT_CONFIG_BASE_NAME: &str = "default";
// e.g. ~/.local/state/knobs/{STATE_FILE_NAME} or
// /var/lib/knobs/{STATE_FILE_NAME}
const STATE_FILE_NAME: &str = "profile.yaml";

// e.g. ~/.config/knobs/profile/{file_name}
fn config_home_with(file_name: &str) -> Option<PathBuf> {
    path::config::home_with(CONFIG_DIR_NAME).map(|mut p| {
        p.push(file_name);
        p
    })
}

// e.g. /etc/knobs/profile/{file_name}
fn config_sys_with(file_name: &str) -> PathBuf {
    let mut p = path::config::sys_with(CONFIG_DIR_NAME);
    p.push(file_name);
    p
}

// Return the list of possible profile paths, in order of preference.
async fn config_paths() -> Vec<PathBuf> {
    static PATHS: OnceCell<Vec<PathBuf>> = OnceCell::const_new();
    async fn paths() -> Vec<PathBuf> {
        let mut paths = vec![];
        if let Some(v) = env::var(CONFIG_ENV_SUFFIX) {
            paths.push(PathBuf::from(v));
        } else {
            let hostname = env::hostname();
            let names = [hostname.as_deref(), Some(DEFAULT_CONFIG_BASE_NAME)];
            for base_name in names.into_iter().flatten() {
                let file_name = format!("{}.yaml", base_name);
                if let Some(p) = config_home_with(&file_name) {
                    paths.push(p);
                }
                let p = config_sys_with(&file_name);
                paths.push(p);
            }
        }
        paths
    }
    PATHS.get_or_init(paths).await.clone()
}

// Return the path to the profile config file.
async fn config_path() -> Option<PathBuf> { config_paths().await.into_iter().find(|p| p.is_file()) }

// Return the path to the profile state file.
async fn state_path() -> Option<PathBuf> {
    if Uid::effective().is_root() {
        Some(path::state::sys_with(STATE_FILE_NAME))
    } else {
        let s = path::state::home_with(STATE_FILE_NAME);
        if s.is_none() {
            log::warn!(
                "ERR knobs w Profile::state_path() Could not determine user state directory"
            );
        }
        s
    }
}
