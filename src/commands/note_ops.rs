use anyhow::Result;

use crate::{db::{Database, Note}, utils::{confirm, format_date_full, parse_tags, resolve_note}};

/// Handles the add command - creates a new note
pub fn handle_add(db: &Database, title: String, content: String, tags: Option<String>) -> Result<()> {
	let tag_vec = parse_tags(tags);
	let note = Note::new(title, content, tag_vec);
	let id = db.create_note(&note)?;
	println!("Note created with ID: {id}");
	Ok(())
}

/// Handles the show command - displays a specific note
pub fn handle_show(db: &Database, id_or_title: &str) -> Result<()> {
	if let Some(note) = db.get_note(resolve_note(db, id_or_title)?)? {
		let sep = "=".repeat(50);
		println!(
			"\n{sep}\nTitle: {}\nTags: {}\nCreated: {}\nUpdated: {}\n{sep}\n\n{}\n",
			note.title,
			note.tags.join(", "),
			format_date_full(&note.created_at),
			format_date_full(&note.updated_at),
			note.content
		);
	}
	Ok(())
}

/// Handles the edit command - modifies an existing note
pub fn handle_edit(
	db: &Database,
	id_or_title: &str,
	title: Option<String>,
	content: Option<String>,
	tags: Option<String>,
) -> Result<()> {
	let id = resolve_note(db, id_or_title)?;
	if let Some(note) = db.get_note(id)? {
		let new_title = title.unwrap_or(note.title);
		let new_content = content.unwrap_or(note.content);
		let new_tags = tags.map(|t| parse_tags(Some(t))).unwrap_or(note.tags);

		db.update_note(id, &new_title, &new_content, &new_tags)?;
		println!("Note {id} updated.");
	}
	Ok(())
}

/// Handles the delete command - removes a note
pub fn handle_delete(db: &Database, id_or_title: &str, yes: bool) -> Result<()> {
	let id = resolve_note(db, id_or_title)?;
	if let Some(note) = db.get_note(id)? {
		println!("Found: [{}] {}", id, note.title);
		if yes || confirm("Delete this note?") {
			db.delete_note(id)?;
			println!("Note {id} deleted.");
		} else {
			println!("Deletion cancelled.");
		}
	}
	Ok(())
}

/// Handles the search command - finds notes by keyword
pub fn handle_search(db: &Database, query: &str) -> Result<()> {
	let notes = db.search_notes(query)?;
	if notes.is_empty() {
		println!("No notes found matching '{query}'.");
	} else {
		println!("Found {} note(s):", notes.len());
		for note in notes {
			if let Some(id) = note.id {
				println!("\n[{id}] {}\nTags: {}", note.title, note.tags.join(", "));
			}
		}
	}
	Ok(())
}
