#[cfg(feature = "cli")]
#[tokio::main]
async fn main() {
    knobs::cli::App::run().await;
}

#[cfg(not(feature = "cli"))]
fn main() {}
