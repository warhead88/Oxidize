//! CLI argument definitions for the gui_studio launcher.

use clap::Parser;

/// 5-Axis CAM Studio — Simulator and toolpath engine launcher.
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to a custom printer TOML configuration file.
    /// If not specified, looks for `printer.toml` in the current directory.
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<String>,

    /// Run without a graphical window (headless mode, for background slicing).
    #[arg(long, default_value_t = false)]
    pub headless: bool,

    /// Enable verbose debug-level logging.
    #[arg(long, default_value_t = false)]
    pub debug: bool,
}
