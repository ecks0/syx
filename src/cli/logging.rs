use tokio::sync::OnceCell;

static LOGGING: OnceCell<()> = OnceCell::const_new();

pub async fn configure() {
    async fn init() {
        use std::io::Write;
        use env_logger::{Builder, Env};
        let env = Env::default()
            .filter_or("KNOBS_LOG", "error")
            .write_style_or("KNOBS_LOG_STYLE", "never");
        Builder::from_env(env)
            .format(|buf, record| {
                writeln!(buf, "{}", record.args())
            })
            .init()
    }
    LOGGING.get_or_init(init).await;
}
