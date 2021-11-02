use std::{collections::HashMap, path::{Path, PathBuf}};
use crate::env;
use crate::path::{profile_user, profile_sys};
use crate::types::Chain;

const DEFAULT_FILE_NAME: &str = "default";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Profile '{profile}' not found in {path}")]
    NoProfile {
        path: String,
        profile: String,
    },

    #[error("{message}: {path}")]
    Parse {
        path: String,
        message: String,
    },

    #[error("{message}: {path}")]
    Io {
        path: String,
        message: String,
    },

    #[error("No profile file exists in {search_paths:#?}")]
    NoFile {
        search_paths: Vec<String>,
    },
}

impl Error {
    fn path_to_str(p: &Path) -> String { p.to_string_lossy().into_owned() }

    fn profile(path: &Path, profile: String) -> Self {
        let path = Self::path_to_str(path);
        Self::NoProfile { path, profile }
    }

    fn parse(path: &Path, message: String) -> Self {
        let path = Self::path_to_str(path);
        Self::Parse { path, message }
    }

    fn io(path: &Path, message: String) -> Self {
        let path = Self::path_to_str(path);
        Self::Io { path, message }
    }

    fn no_file(search_paths: Vec<PathBuf>) -> Self {
        let search_paths = search_paths.into_iter().map(|p| Self::path_to_str(&p)).collect();
        Self::NoFile { search_paths }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

// Return a list of possible paths for the profile file.
pub fn paths() -> Vec<PathBuf> {
    let mut res = vec![];
    if let Some(v) = env::var("PROFILE_PATH") {
        res.push(PathBuf::from(v));
    } else {
        for base_name in [env::hostname().as_deref(), Some(DEFAULT_FILE_NAME)].into_iter().flatten() {
            let file_name = format!("{}.yaml", base_name);
            if let Some(p) = profile_user(&file_name) { res.push(p); }
            let p = profile_sys(&file_name);
            res.push(p);
        }
    }
    res
}

// Return the path to the profile file.
pub fn path() -> Option<PathBuf> {
    paths()
        .into_iter()
        .find(|p| p.is_file())
}

// Load the given profile name from the profile file.
pub async fn read(name: &str) -> Result<Chain> {
    let path = if let Some(p) = path() { p } else {
        return Err(Error::no_file(paths()));
    };
    log::debug!("Reading profiles from {}", path.display());
    match tokio::fs::read_to_string(&path).await {
        Ok(s) =>
            match serde_yaml::from_str::<HashMap<String, Chain>>(&s) {
                Ok(p) =>
                    match p.into_iter().find(|(n, _)| n == name).map(|(_, c)| c) {
                        Some(c) => Ok(c),
                        None => Err(Error::profile(&path, name.to_string())),
                    },
                Err(e) => Err(Error::parse(&path, e.to_string())),
            },
        Err(e) => Err(Error::io(&path, e.to_string())),
    }
}
