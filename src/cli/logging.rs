use fern::Dispatch;
use log::LevelFilter;
use crate::Result;

static DEBUG_VAR: &str = "KNOBS_DEBUG";

fn debug() -> bool {
    if let Ok(v) = std::env::var(DEBUG_VAR) {
        !v.is_empty() && v != "0" && v.to_lowercase() != "false"
    } else {
        false
    }
}

pub fn configure(verbose: bool) -> Result<()> {
    let level =
        if debug() {
            LevelFilter::Debug
        } else if verbose {
            LevelFilter::Warn
        } else {
            return Ok(());
        };
    Dispatch::new()
        .format(|out, message, _record| {
            out.finish(format_args!("{}", message))
        })
        .level(level)
        .chain(std::io::stderr())
        .apply()?;
    Ok(())
}
