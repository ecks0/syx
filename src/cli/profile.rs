use std::collections::HashMap;
use std::fmt::Display;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tokio::fs::{create_dir_all, read_to_string, write};
use tokio::io::ErrorKind as IoErrorKind;

use crate::cli::values::Values;

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
    fn de<P: Into<PathBuf>, S: Display>(path: P, message: S) -> Self {
        let path = path.into();
        let message = message.to_string();
        Self::De { path, message }
    }

    fn io<P: Into<PathBuf>, S: Display>(path: P, message: S) -> Self {
        let path = path.into();
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

    fn profile_missing<P: Into<PathBuf>, S: Display>(path: P, profile: S) -> Self {
        let path = path.into();
        let profile = profile.to_string();
        Self::ProfileMissing { path, profile }
    }

    fn state_corrupt<P: Into<PathBuf>>(path: P) -> Self {
        let path = path.into();
        Self::StateCorrupt { path }
    }

    fn state_missing<P: Into<PathBuf>>(path: P) -> Self {
        let path = path.into();
        Self::StateMissing { path }
    }

    fn state_path_missing<S: Display>(activity: S) -> Self {
        let activity = activity.to_string();
        Self::StatePathMissing { activity }
    }

    fn se<S: Display>(message: S) -> Self {
        let message = message.to_string();
        Self::Se { message }
    }
}

pub(in crate::cli) type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Profile {
    name: String,
    path: PathBuf,
}

impl Profile {
    const RECENT_PROFILE: &'static str = "_";

    pub(in crate::cli) async fn new<S: Into<String>>(name: S) -> Result<Self> {
        let name = name.into();
        if Self::RECENT_PROFILE == name.as_str() {
            Self::recent().await
        } else {
            let path = match path::config_path().await {
                Some(path) => path,
                None => return Err(Error::config_missing(path::config_paths().await)),
            };
            let s = Self { name, path };
            Ok(s)
        }
    }

    pub(in crate::cli) async fn values(&self) -> Result<Vec<Values>> {
        log::debug!(
            "Loading profile '{}' from {}",
            self.name,
            self.path.display(),
        );
        match read_to_string(&self.path).await {
            Ok(s) => match serde_yaml::from_str::<HashMap<String, Vec<Values>>>(&s) {
                Ok(cf) => match cf.into_iter().find(|(k, _)| k == &self.name) {
                    Some((_, v)) => Ok(v),
                    None => Err(Error::profile_missing(&self.path, &self.name)),
                },
                Err(e) => Err(Error::de(&self.path, e)),
            },
            Err(e) => match e.kind() {
                IoErrorKind::NotFound => Err(Error::config_missing(path::config_paths().await)),
                _ => Err(Error::io(&self.path, e)),
            },
        }
    }

    async fn recent() -> Result<Self> {
        match path::state_path().await {
            Some(p) => match read_to_string(&p).await {
                Ok(s) => serde_yaml::from_str(&s).map_err(|_| Error::state_corrupt(p)),
                Err(e) => match e.kind() {
                    IoErrorKind::NotFound => Err(Error::state_missing(p)),
                    _ => Err(Error::io(p, e)),
                },
            },
            None => Err(Error::state_path_missing("Read recent profile")),
        }
    }

    pub(in crate::cli) async fn set_recent(&self) -> Result<()> {
        match path::state_path().await {
            Some(p) => {
                if let Some(parent) = p.parent() {
                    if !parent.is_dir() {
                        create_dir_all(parent)
                            .await
                            .map_err(|e| Error::io(parent, e))?;
                    }
                }
                let s = serde_yaml::to_string(self).map_err(Error::se)?;
                write(&p, s.as_bytes()).await.map_err(|e| Error::io(p, e))
            },
            None => Err(Error::state_path_missing("Write recent profile")),
        }
    }
}

pub(in crate::cli) mod path {
    use std::path::PathBuf;

    use nix::unistd::Uid;
    use tokio::sync::OnceCell;

    use crate::cli::{env, path};

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
    pub(in crate::cli) async fn config_paths() -> Vec<PathBuf> {
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
    pub(in crate::cli) async fn config_path() -> Option<PathBuf> {
        config_paths().await.into_iter().find(|p| p.is_file())
    }

    // Return the path to the profile state file.
    pub(in crate::cli) async fn state_path() -> Option<PathBuf> {
        if Uid::effective().is_root() {
            Some(path::state::sys_with(STATE_FILE_NAME))
        } else {
            path::state::home_with(STATE_FILE_NAME)
        }
    }
}
