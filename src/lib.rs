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

use crate::cli::Cli;

pub fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    run_with_args(&args)
}

pub fn run_with_args(args: &[String]) -> Result<()> {
    let cli = match Cli::from_args(args) {
        Ok(cli) => cli,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        },
    };
    cli.apply();
    cli.show();
    Ok(())
}
