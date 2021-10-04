mod cli;
mod policy;
mod table;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    LogSetLogger(#[from] log::SetLoggerError),

    #[error("Parse error: {flag}: {msg}")]
    Parse {
        flag: &'static str,
        msg: &'static str,
    },
}

impl Error {
    fn parse(flag: &'static str, msg: &'static str) -> Self { Self::Parse { flag, msg } }
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn run() -> Result<()> {
    use std::env::args;
    use crate::{cli::Cli, policy::Policy};

    let args: Vec<String> = args().collect();
    let cli = match Cli::from_args(&args) {
        Ok(cli) => cli,
        Err(err) =>
            match err {
                Error::Parse { flag, msg } => {
                    eprintln!("Error: {} {}", flag, msg);
                    std::process::exit(1);
                },
                _ => return Err(err),
            },
    };
    let policy = Policy::from_cli(&cli);
    policy.apply();
    if let Some(s) = table::format() {
        println!("\n{}", s);
    }
    Ok(())
}
