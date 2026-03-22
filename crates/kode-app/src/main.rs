mod app;
mod cli;

use tracing_subscriber::EnvFilter;

fn main() {
    let args = cli::parse();

    if args.gpu {
        // GPU mode: standard logging
        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
            )
            .init();

        tracing::info!("Starting kode v{}", env!("CARGO_PKG_VERSION"));

        if let Err(e) = app::run(args) {
            tracing::error!("Fatal error: {}", e);
            std::process::exit(1);
        }
    } else {
        // TUI mode (default): log to stderr to avoid terminal corruption
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::new("warn"))
            .with_writer(std::io::stderr)
            .init();

        if let Err(e) = app::run_tui(args) {
            eprintln!("Fatal error: {}", e);
            std::process::exit(1);
        }
    }
}
