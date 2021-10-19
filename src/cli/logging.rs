use env_logger::{Builder, Env};
use std::io::Write;

pub fn configure() {
    let env = Env::default()
        .filter_or("KNOBS_LOG", "error")
        .write_style_or("KNOBS_LOG_STYLE", "never");

    Builder::from_env(env)
        .format(|buf, record| {
            writeln!(buf, "{}", record.args())
        })
        .init();
}
