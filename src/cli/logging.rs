use fern::Dispatch;
use crate::Result;

pub fn enable() -> Result<()> {
    Dispatch::new()
        .format(|out, message, _record| {
            out.finish(format_args!("{}", message))
        })
        .level(log::LevelFilter::Warn)
        .chain(std::io::stderr())
        .apply()?;
    Ok(())
}
