use std::path::PathBuf;
use super::NAME;

// Sub-directory containing profile files.
const PROFILE: &str = "profile";

// Name of state file.
// const STATE: &'static str = "state.yaml";

// e.g. ~/.config/knobs
fn config_user() -> Option<PathBuf> {
    dirs::config_dir()
        .map(|mut p| {
            p.push(NAME);
            p
        })
}

// /etc/knobs
fn config_sys() -> PathBuf {
    let mut p = PathBuf::new();
    p.push("/etc");
    p.push(NAME);
    p
}

// e.g. ~/.config/knobs/profile/<file_name>
pub fn profile_user(file_name: &str) -> Option<PathBuf> {
    config_user()
        .map(|mut p| {
            p.push(PROFILE);
            p.push(file_name);
            p
        })
}

// /etc/knobs/profile/<file_name>
pub fn profile_sys(file_name: &str) -> PathBuf {
    let mut p = config_sys();
    p.push(PROFILE);
    p.push(file_name);
    p
}

// e.g. ~/.local/state/knobs/state.yaml
// pub fn state() -> Option<PathBuf> {
//     dirs::state_dir()
//         .map(|mut p| {
//             p.push(NAME);
//             p.push(STATE);
//             p
//         })
// }
