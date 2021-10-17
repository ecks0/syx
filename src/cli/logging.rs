use fern::Dispatch;
use log::LevelFilter;

const DEBUG_VAR: &str = "KNOBS_DEBUG";

fn debug() -> bool {
    if let Ok(v) = std::env::var(DEBUG_VAR) {
        !v.is_empty() && v != "0" && v.to_lowercase() != "false"
    } else {
        false
    }
}

pub fn configure(verbose: bool) {
    let level =
        if debug() {
            LevelFilter::Debug
        } else if verbose {
            LevelFilter::Warn
        } else {
            LevelFilter::Error
        };
    if let Err(err) = Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!("{:>5} {}", record.level(), message))
        })
        .level(level)
        .chain(std::io::stderr())
        .apply()
    {
        eprintln!("Log configuration error: {}", err);
    }
}
