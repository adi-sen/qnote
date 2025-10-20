use std::{env, fs, io::{self, Write}, path::PathBuf, process::Command};

use anyhow::{Context, Result};
use ratatui::crossterm::{execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode}};

use crate::{config::EditorConfig, db::Note, utils::parse_markdown_file};

/// Returns the user's preferred editor from environment variables or config.
/// Priority: config.default_editor > $EDITOR > vi
fn get_editor(config: &EditorConfig) -> String {
	config.default_editor.clone().unwrap_or_else(|| env::var("EDITOR").unwrap_or_else(|_| "vi".to_string()))
}

/// Returns a consistent temp file path for qnote editing.
/// Reusing the same path avoids filesystem overhead and is faster than creating
/// new files.
fn get_temp_path() -> PathBuf { env::temp_dir().join("qnote-edit.md") }

/// Opens the user's editor with an empty template for creating a new note.
/// Returns None if the user cancels or creates an empty note.
/// Returns Some((title, content, tags)) if a valid note is created.
pub fn open_editor_for_new_note(config: &EditorConfig) -> Result<Option<(String, String, Vec<String>)>> {
	let temp_path = get_temp_path();

	// Create file with appropriate permissions
	#[cfg(unix)]
	{
		use std::os::unix::fs::OpenOptionsExt;
		let mode = if config.secure_temp_files { 0o600 } else { 0o644 };
		fs::OpenOptions::new().write(true).create(true).truncate(true).mode(mode).open(&temp_path)?;
	}
	#[cfg(not(unix))]
	{
		let mut file = fs::File::create(&temp_path)?;
		file.flush()?;
		drop(file);
	}

	open_editor(&temp_path, config)?;

	// Read and parse the edited content
	let content = fs::read_to_string(&temp_path)?;

	Ok(parse_markdown_file(&content))
}

/// Opens the user's editor with an existing note's content pre-filled.
/// Note format: title on line 1, hashtags on line 2 (if any), content after
/// blank line. Returns None if the user cancels or deletes all content.
/// Returns Some((title, content, tags)) if the note is successfully edited.
pub fn open_editor_for_edit(note: &Note, config: &EditorConfig) -> Result<Option<(String, String, Vec<String>)>> {
	let temp_path = get_temp_path();

	// Create file with appropriate permissions and use BufWriter for better I/O
	// performance
	#[cfg(unix)]
	let file = {
		use std::os::unix::fs::OpenOptionsExt;
		let mode = if config.secure_temp_files { 0o600 } else { 0o644 };
		fs::OpenOptions::new().write(true).create(true).truncate(true).mode(mode).open(&temp_path)?
	};
	#[cfg(not(unix))]
	let file = fs::File::create(&temp_path)?;
	let mut writer = io::BufWriter::new(file);

	// Write title
	write!(writer, "{}", note.title)?;

	// Write tags if present (optimized: avoid intermediate Vec allocation)
	if !note.tags.is_empty() {
		writer.write_all(b"\n")?;
		for (i, tag) in note.tags.iter().enumerate() {
			if i > 0 {
				writer.write_all(b" ")?;
			}
			write!(writer, "@{tag}")?;
		}
	}

	// Write content if present
	if !note.content.is_empty() {
		writer.write_all(b"\n\n")?;
		write!(writer, "{}", note.content)?;
	}

	writer.flush()?;
	drop(writer);

	open_editor(&temp_path, config)?;

	// Read back and parse the edited content
	let content = fs::read_to_string(&temp_path)?;

	Ok(parse_markdown_file(&content))
}

/// Opens the user's preferred editor for the given file path.
/// Temporarily exits the TUI alternate screen and raw mode, then restores them
/// after editing. This allows the editor to function normally without
/// interference from the TUI.
fn open_editor(path: &std::path::Path, config: &EditorConfig) -> Result<()> {
	let editor = get_editor(config);

	// Temporarily exit TUI mode so editor can take over the terminal
	disable_raw_mode()?;
	execute!(io::stdout(), LeaveAlternateScreen)?;

	let status = Command::new(&editor).arg(path).status().context(format!("Failed to open editor: {editor}"))?;

	// Restore TUI mode after editor closes
	execute!(io::stdout(), EnterAlternateScreen)?;
	enable_raw_mode()?;

	if !status.success() {
		anyhow::bail!("Editor exited with non-zero status");
	}

	Ok(())
}
