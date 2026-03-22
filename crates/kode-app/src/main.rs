mod app;
mod cli;

use tracing_subscriber::EnvFilter;

fn main() {
    let args = cli::parse();

    // Initialize logging — in TUI mode, log to file to avoid corrupting the terminal
    if args.tui {
        // Suppress tracing output when in TUI mode
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::new("warn"))
            .with_writer(std::io::stderr)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
            )
            .init();
    }

    tracing::info!("Starting kode v{}", env!("CARGO_PKG_VERSION"));

    if args.tui {
        if let Err(e) = app::run_tui(args) {
            eprintln!("Fatal error: {}", e);
            std::process::exit(1);
        }
    } else {
        if let Err(e) = app::run(args) {
            tracing::error!("Fatal error: {}", e);
            std::process::exit(1);
        }
    }
}
