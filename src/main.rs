mod cli;
mod db;
mod tui;
mod utils;

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use cli::{Cli, Commands};
use db::Database;

/// Returns platform-specific database path, creating directory if needed.
fn get_db_path() -> Result<PathBuf> {
	let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
	path.push("qnote");
	std::fs::create_dir_all(&path).context("Failed to create qnote data directory")?;
	path.push("notes.db");
	Ok(path)
}

fn main() -> Result<()> {
	let db_path = get_db_path()?;
	let db_path_str = db_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid database path"))?;
	let db = Database::new(db_path_str)?;
	let cli = Cli::parse();

	match cli.command {
		Some(Commands::Tui) | None => tui::run_tui(db)?,
		Some(cmd) => cli::handle_command(&db, cmd)?,
	}

	Ok(())
}
