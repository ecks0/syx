use std::path::PathBuf;

use nix::unistd::Uid;
use tokio::sync::OnceCell;

use crate::{env, NAME};

// Build config paths, e.g. in ~/.config or /etc
mod config {
    use super::*;

    const DIR_DEFAULT: &str = "/etc";

    const DIR: Option<&'static str> = option_env!("KNOBS_SYS_CONFIG_DIR");

    // e.g. ~/.config/knobs
    pub(super) fn home() -> Option<PathBuf> {
        dirs::config_dir().map(|mut p| {
            p.push(NAME);
            p
        })
    }

    // e.g. /etc/knobs
    pub(super) fn sys() -> PathBuf {
        let dir = DIR.unwrap_or(DIR_DEFAULT);
        let mut p = PathBuf::new();
        p.push(dir);
        p.push(NAME);
        p
    }

    pub(super) fn home_with(file_name: &str) -> Option<PathBuf> {
        home().map(|mut p| {
            p.push(file_name);
            p
        })
    }

    pub(super) fn sys_with(file_name: &str) -> PathBuf {
        let mut p = sys();
        p.push(file_name);
        p
    }
}

// Build state paths, e.g. in ~/.local/state or /var/lib
mod state {
    use super::*;

    const DIR_DEFAULT: &str = "/var/lib";

    const DIR: Option<&'static str> = option_env!("KNOBS_SYS_STATE_DIR");

    // e.g. ~/.local/state/knobs
    pub(super) fn home() -> Option<PathBuf> {
        dirs::state_dir().map(|mut p| {
            p.push(NAME);
            p
        })
    }

    // e.g. /var/lib/knobs
    pub(super) fn sys() -> PathBuf {
        let dir = DIR.unwrap_or(DIR_DEFAULT);
        let mut p = PathBuf::new();
        p.push(dir);
        p.push(NAME);
        p
    }

    pub(super) fn home_with(file_name: &str) -> Option<PathBuf> {
        home().map(|mut p| {
            p.push(file_name);
            p
        })
    }

    pub(super) fn sys_with(file_name: &str) -> PathBuf {
        let mut p = sys();
        p.push(file_name);
        p
    }
}

// Build profile config and state paths.
pub(crate) mod profile {
    use super::*;

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
        config::home_with(CONFIG_DIR_NAME).map(|mut p| {
            p.push(file_name);
            p
        })
    }

    // e.g. /etc/knobs/profile/{file_name}
    fn config_sys_with(file_name: &str) -> PathBuf {
        let mut p = config::sys_with(CONFIG_DIR_NAME);
        p.push(file_name);
        p
    }

    // Return the list of possible profile paths, in order of preference.
    pub(crate) async fn config_paths() -> Vec<PathBuf> {
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
    pub(crate) async fn config_path() -> Option<PathBuf> {
        config_paths().await.into_iter().find(|p| p.is_file())
    }

    // Return the path to the profile state file.
    pub(crate) async fn state_path() -> Option<PathBuf> {
        if Uid::effective().is_root() {
            Some(state::sys_with(STATE_FILE_NAME))
        } else {
            state::home_with(STATE_FILE_NAME)
        }
    }
}
