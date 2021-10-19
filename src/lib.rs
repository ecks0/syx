mod cli;
mod policy;
mod format;

pub fn run() {
    let args: Vec<String> = std::env::args().collect();
    run_with_args(&args)
}

pub fn run_with_args(args: &[String]) {
    use crate::cli::Cli;

    match Cli::from_args(args) {
        Ok(cli) => cli.run(),
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        },
    }
}
