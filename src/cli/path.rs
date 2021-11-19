use std::path::PathBuf;

use crate::cli::NAME;

// Build config paths, e.g. in ~/.config or /etc
pub(in crate::cli) mod config {
    use super::*;

    const DIR_DEFAULT: &str = "/etc";

    const DIR: Option<&'static str> = option_env!("KNOBS_SYS_CONFIG_DIR");

    // e.g. ~/.config/knobs
    pub(in crate::cli) fn home() -> Option<PathBuf> {
        dirs::config_dir().map(|mut p| {
            p.push(NAME);
            p
        })
    }

    // e.g. /etc/knobs
    pub(in crate::cli) fn sys() -> PathBuf {
        let dir = DIR.unwrap_or(DIR_DEFAULT);
        let mut p = PathBuf::new();
        p.push(dir);
        p.push(NAME);
        p
    }

    pub(in crate::cli) fn home_with(file_name: &str) -> Option<PathBuf> {
        home().map(|mut p| {
            p.push(file_name);
            p
        })
    }

    pub(in crate::cli) fn sys_with(file_name: &str) -> PathBuf {
        let mut p = sys();
        p.push(file_name);
        p
    }
}

// Build state paths, e.g. in ~/.local/state or /var/lib
pub(in crate::cli) mod state {
    use super::*;

    const DIR_DEFAULT: &str = "/var/lib";

    const DIR: Option<&'static str> = option_env!("KNOBS_SYS_STATE_DIR");

    // e.g. ~/.local/state/knobs
    pub(in crate::cli) fn home() -> Option<PathBuf> {
        dirs::state_dir().map(|mut p| {
            p.push(NAME);
            p
        })
    }

    // e.g. /var/lib/knobs
    pub(in crate::cli) fn sys() -> PathBuf {
        let dir = DIR.unwrap_or(DIR_DEFAULT);
        let mut p = PathBuf::new();
        p.push(dir);
        p.push(NAME);
        p
    }

    pub(in crate::cli) fn home_with(file_name: &str) -> Option<PathBuf> {
        home().map(|mut p| {
            p.push(file_name);
            p
        })
    }

    pub(in crate::cli) fn sys_with(file_name: &str) -> PathBuf {
        let mut p = sys();
        p.push(file_name);
        p
    }
}
