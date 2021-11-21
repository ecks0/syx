mod app;
mod de;
mod env;
mod format;
mod group;
mod logging;
mod parse;
mod parser;
mod path;
mod policy;
mod profile;
mod sampler;

use std::fmt::Display;

pub use clap::{Error as ClapError, ErrorKind as ClapErrorKind};
pub use profile::Error as ProfileError;
pub use tokio::io::{Error as IoError, ErrorKind as IoErrorKind};

pub use crate::cli::app::{App, Cli};

const NAME: &str = "knobs";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Clap(#[from] ClapError),

    #[error(transparent)]
    Format(IoError),

    #[error("--{flag}: {message}")]
    ParseFlag { flag: String, message: String },

    #[error("{0}")]
    ParseValue(String),

    #[error(transparent)]
    Profile(#[from] ProfileError),
}

impl Error {
    fn parse_flag<S: Display>(flag: &str, message: S) -> Self {
        let flag = flag.to_string();
        let message = message.to_string();
        Self::ParseFlag { flag, message }
    }

    fn parse_value<S: Display>(message: S) -> Self {
        let message = message.to_string();
        Self::ParseValue(message)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
