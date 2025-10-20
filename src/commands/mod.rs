mod config;
mod io;
mod list;
mod note_ops;

use anyhow::Result;
pub use config::handle_config;
pub use io::{handle_export, handle_import};
pub use list::{handle_list, handle_stats, handle_tags};
pub use note_ops::{handle_add, handle_delete, handle_edit, handle_search, handle_show};

use crate::{cli::Commands, db::Database};

/// Dispatches CLI commands to their respective handlers
pub fn handle_command(db: &Database, cmd: Commands) -> Result<()> {
	match cmd {
		Commands::Add { title, content, tags } => handle_add(db, title, content, tags),
		Commands::List { tag, oneline, sort, limit } => handle_list(db, tag, oneline, sort, limit),
		Commands::Show { id_or_title } => handle_show(db, &id_or_title),
		Commands::Edit { id_or_title, title, content, tags } => handle_edit(db, &id_or_title, title, content, tags),
		Commands::Delete { id_or_title, yes } => handle_delete(db, &id_or_title, yes),
		Commands::Search { query } => handle_search(db, &query),
		Commands::Export { id_or_title, output } => handle_export(db, &id_or_title, output),
		Commands::Import { files } => handle_import(db, &files),
		Commands::Tags => handle_tags(db),
		Commands::Stats => handle_stats(db),
		Commands::Tui => Ok(()), // Never reached - TUI is handled in main.rs
		Commands::Config { show } => handle_config(show),
	}
}
