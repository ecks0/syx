use tokio::sync::OnceCell;

use crate::env::var_name;

pub(crate) async fn configure() {
    static LOGGING: OnceCell<()> = OnceCell::const_new();
    async fn init() {
        use std::io::Write;

        use env_logger::{Builder, Env};
        let env = Env::default()
            .filter_or(var_name("LOG"), "error")
            .write_style_or(var_name("LOG_STYLE"), "never");
        Builder::from_env(env).format(|buf, record| writeln!(buf, "{}", record.args())).init()
    }
    LOGGING.get_or_init(init).await;
}
