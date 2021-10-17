mod cli;
mod policy;
mod table;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error: {flag} {msg}")]
    Parse {
        flag: &'static str,
        msg: &'static str,
    },
}

impl Error {
    fn parse(flag: &'static str, msg: &'static str) -> Self { Self::Parse { flag, msg } }
}

pub type Result<T> = std::result::Result<T, Error>;

use std::env::args;
use crate::{cli::Cli, policy::Policy};

pub fn run() -> Result<()> {
    let args: Vec<String> = args().collect();
    let cli = match Cli::from_args(&args) {
        Ok(cli) => cli,
        Err(err) =>
            match err {
                Error::Parse { .. } => {
                    eprintln!("{}", err);
                    std::process::exit(1);
                },
            },
    };
    let policy = Policy::from_cli(&cli);
    policy.apply();
    if let Some(s) = table::format() {
        println!("\n{}", s);
    }
    Ok(())
}
