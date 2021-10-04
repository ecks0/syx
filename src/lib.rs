pub mod cli;
pub mod policy;
pub mod table;
mod timer;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Clap(#[from] clap::Error),

    #[error(transparent)]
    LogSetLogger(#[from] log::SetLoggerError),

    #[error("Parse error: {flag}: {msg}")]
    Parse {
        flag: &'static str,
        msg: &'static str,
    },

    #[error(transparent)]
    Zysfs(#[from] zysfs::io::blocking::Error),
}

impl Error {
    fn parse(flag: &'static str, msg: &'static str) -> Self { Self::Parse { flag, msg } }
}

pub type Result<T> = std::result::Result<T, Error>;
