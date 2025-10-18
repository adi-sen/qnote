mod cli;
mod db;
mod tui;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use db::Database;
use std::path::PathBuf;

/// Determines the database file path based on the platform.
///
/// Returns the platform-specific data directory:
/// - Linux: ~/.local/share/qnote/notes.db
/// - macOS: ~/Library/Application Support/qnote/notes.db
/// - Windows: C:\Users\<User>\AppData\Roaming\qnote\notes.db
///
/// Creates the qnote directory if it doesn't exist.
/// Falls back to current directory if the system data directory cannot be determined.
fn get_db_path() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("qnote");
    std::fs::create_dir_all(&path).ok();
    path.push("notes.db");
    path
}

fn main() -> Result<()> {
    // Initialize database connection
    let db_path = get_db_path();
    let db = Database::new(db_path.to_str().unwrap())?;

    // Parse command-line arguments
    let cli = Cli::parse();

    // Route to TUI or CLI command handler
    match cli.command {
        // No command or explicit 'tui' command: launch interactive TUI
        Some(Commands::Tui) | None => {
            tui::run_tui(db)?;
        }
        // Execute CLI command (add, list, edit, etc.)
        Some(cmd) => {
            cli::handle_command(&db, cmd)?;
        }
    }

    Ok(())
}
