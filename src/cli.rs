//! Command-line interface definitions for qnote.
//!
//! Defines CLI argument structures and command enums.
//! Command implementations are in the `commands` module.

use clap::{Parser, Subcommand};

/// Main CLI structure parsed by clap.
#[derive(Parser)]
#[command(name = "qnote")]
#[command(about = "A quick note-taking app", long_about = None)]
pub struct Cli {
	#[command(subcommand)]
	pub command: Option<Commands>,
}

/// Sort order for list command
#[derive(Clone, Copy, clap::ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum SortBy {
	Updated,
	Created,
	Title,
}

/// Available CLI commands.
#[derive(Subcommand)]
pub enum Commands {
	/// Add a new note with title, content, and optional tags
	Add {
		title:   String,
		content: String,
		#[arg(short, long)]
		tags:    Option<String>,
	},
	/// List all notes
	List {
		#[arg(short, long)]
		tag:     Option<String>,
		#[arg(short, long)]
		oneline: bool,
		#[arg(short, long, default_value = "updated")]
		sort:    SortBy,
		#[arg(short, long)]
		limit:   Option<usize>,
	},
	/// Show a specific note (by ID or title pattern)
	Show { id_or_title: String },
	/// Edit a note (by ID or title pattern)
	Edit {
		id_or_title: String,
		#[arg(short, long)]
		title:       Option<String>,
		#[arg(short, long)]
		content:     Option<String>,
		#[arg(short = 'g', long)]
		tags:        Option<String>,
	},
	/// Delete a note (by ID or title pattern)
	Delete {
		id_or_title: String,
		#[arg(short, long)]
		yes:         bool,
	},
	/// Search notes by keyword
	Search { query: String },
	/// Export a note to a markdown file
	Export {
		id_or_title: String,
		#[arg(short, long)]
		output:      Option<String>,
	},
	/// Import notes from markdown files
	Import { files: Vec<String> },
	/// List all tags with note counts
	Tags,
	/// Show statistics about notes
	Stats,
	/// Open TUI interface
	Tui,
	/// Generate a default configuration file
	Config {
		#[arg(short, long)]
		show: bool,
	},
}
