use std::{fs, path::Path};

use anyhow::Result;

use crate::{db::{Database, Note}, utils::{note_to_markdown, parse_markdown_file, resolve_note, sanitize_filename}};

/// Handles the export command - exports a note to markdown file
pub fn handle_export(db: &Database, id_or_title: &str, output: Option<String>) -> Result<()> {
	let id = resolve_note(db, id_or_title)?;
	if let Some(note) = db.get_note(id)? {
		let filename = output.unwrap_or_else(|| format!("{}.md", sanitize_filename(&note.title)));
		let content = note_to_markdown(&note);

		fs::write(&filename, content)?;
		println!("Exported to: {filename}");
	}
	Ok(())
}

/// Handles the import command - imports notes from markdown files
pub fn handle_import(db: &Database, files: &[String]) -> Result<()> {
	let mut imported = 0;
	for file_path in files {
		let path = Path::new(file_path);
		if !path.exists() {
			eprintln!("Warning: File not found: {file_path}");
			continue;
		}

		let content = fs::read_to_string(path)?;

		if let Some((title, note_content, tags)) = parse_markdown_file(&content) {
			let note = Note::new(title, note_content, tags);
			db.create_note(&note)?;
			imported += 1;
			let display_path = path.display();
			println!("Imported: {display_path}");
		} else {
			eprintln!("Warning: Could not parse: {file_path}");
		}
	}
	println!("\nImported {imported} note(s)");
	Ok(())
}
