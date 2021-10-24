mod cli;
mod policy;
mod format;

pub fn run() {
    match cli::Cli::new() {
        Ok(cli) => cli.run(),
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        },
    }
}
