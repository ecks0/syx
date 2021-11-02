use tokio::sync::OnceCell;
use std::time::{Duration, Instant};

// Runtime counter.
static COUNTER: OnceCell<Instant> = OnceCell::const_new();

pub async fn get() -> Instant {
    async fn start() -> Instant { Instant::now() }
    *COUNTER.get_or_init(start).await
}

pub async fn delta() -> Duration {
    let then = get().await;
    Instant::now() - then
}

