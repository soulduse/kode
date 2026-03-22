use clap::Parser;
use std::path::PathBuf;

/// Kode — A fast, Rust-native IDE combining tmux workflow with IntelliJ intelligence.
#[derive(Parser, Debug)]
#[command(name = "kode", version, about)]
pub struct Args {
    /// File(s) to open
    pub files: Vec<PathBuf>,

    /// Use TUI mode instead of GPU rendering
    #[arg(long)]
    pub tui: bool,

    /// Path to configuration file
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Restore previous session
    #[arg(long)]
    pub restore: bool,
}

pub fn parse() -> Args {
    Args::parse()
}
