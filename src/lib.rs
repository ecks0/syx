mod cli;
mod policy;
mod table;
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

pub fn run() -> Result<()> {
    let mut t = timer::Timer::start();

    let args: Vec<String> = std::env::args().collect();
    t.end("Collect args");

    let cli = match cli::Cli::from_args(&args) {
        Ok(cli) => cli,
        Err(err) =>
            match err {
                Error::Parse { flag, msg } => {
                    println!("Error: {} {}", flag, msg);
                    std::process::exit(1);
                },
                _ => return Err(err),
            },
    };
    t.end("Build cli");

    let policy = policy::Policy::from_cli(&cli);
    t.end("Build policy");

    policy.apply();
    t.end("Apply policy");

    let s = table::format();
    t.end("Format table");

    if let Some(s) = s { println!("\n{}", s); }

    Ok(())
}
