use std::path::PathBuf;

use super::NAME;

pub(crate) mod config {
    use super::*;

    const DIR_DEFAULT: &str = "/etc";

    const DIR: Option<&'static str> = option_env!("KNOBS_SYS_CONFIG_DIR");

    // e.g. ~/.config/knobs
    pub(crate) fn home() -> Option<PathBuf> {
        dirs::config_dir().map(|mut p| {
            p.push(NAME);
            p
        })
    }

    // e.g. /etc/knobs
    pub(crate) fn sys() -> PathBuf {
        let dir = DIR.unwrap_or(DIR_DEFAULT);
        let mut p = PathBuf::new();
        p.push(dir);
        p.push(NAME);
        p
    }

    pub(crate) fn home_with(file_name: &str) -> Option<PathBuf> {
        home().map(|mut p| {
            p.push(file_name);
            p
        })
    }

    pub(crate) fn sys_with(file_name: &str) -> PathBuf {
        let mut p = sys();
        p.push(file_name);
        p
    }
}

pub(crate) mod state {
    use super::*;

    const DIR_DEFAULT: &str = "/var/lib";

    const DIR: Option<&'static str> = option_env!("KNOBS_SYS_STATE_DIR");

    // e.g. ~/.local/state/knobs
    pub(crate) fn home() -> Option<PathBuf> {
        dirs::state_dir().map(|mut p| {
            p.push(NAME);
            p
        })
    }

    // e.g. /var/lib/knobs
    pub(crate) fn sys() -> PathBuf {
        let dir = DIR.unwrap_or(DIR_DEFAULT);
        let mut p = PathBuf::new();
        p.push(dir);
        p.push(NAME);
        p
    }

    pub(crate) fn home_with(file_name: &str) -> Option<PathBuf> {
        home().map(|mut p| {
            p.push(file_name);
            p
        })
    }

    pub(crate) fn sys_with(file_name: &str) -> PathBuf {
        let mut p = sys();
        p.push(file_name);
        p
    }
}
