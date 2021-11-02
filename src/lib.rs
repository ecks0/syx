pub(crate) mod cli;
pub(crate) mod data;
pub(crate) mod de;
pub(crate) mod env;
pub(crate) mod format;
pub(crate) mod lazy;
pub(crate) mod logging;
pub(crate) mod parse;
pub(crate) mod path;
pub(crate) mod policy;
pub(crate) mod profile;
pub(crate) mod types;

pub use clap::{
    Error as ClapError,
    ErrorKind as ClapErrorKind,
};

pub use profile::Error as ProfileError;

pub use tokio::io::{
    Error as IoError,
    ErrorKind as IoErrorKind,
};

pub use cli::App;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Clap(#[from] ClapError),

    #[error(transparent)]
    Format(IoError),

    #[error("--{flag}: {message}")]
    ParseFlag {
        flag: String,
        message: String,
    },

    #[error("{0}")]
    ParseValue(String),

    #[error(transparent)]
    Profile(#[from] ProfileError),
}

impl Error {
    fn parse_flag(flag: &str, message: String) -> Self {
        let flag = flag.to_string();
        Self::ParseFlag {
            flag,
            message,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

const NAME: &str = "knobs";
