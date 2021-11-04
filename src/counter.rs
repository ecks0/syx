use std::time::{Duration, Instant};
use tokio::sync::OnceCell;

pub async fn get() -> Instant {
    static COUNTER: OnceCell<Instant> = OnceCell::const_new();
    async fn start() -> Instant { Instant::now() }
    *COUNTER.get_or_init(start).await
}

pub async fn delta() -> Duration {
    let then = get().await;
    Instant::now() - then
}
