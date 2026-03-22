mod app;
mod cli;

use tracing_subscriber::EnvFilter;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let args = cli::parse();

    tracing::info!("Starting kode v{}", env!("CARGO_PKG_VERSION"));

    if let Err(e) = app::run(args) {
        tracing::error!("Fatal error: {}", e);
        std::process::exit(1);
    }
}
