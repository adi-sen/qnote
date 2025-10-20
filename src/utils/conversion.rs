//! Conversion utilities for notes and database operations.

use anyhow::Result;

use crate::db::{Database, Note};

/// Formats a note as markdown content with title, tags, and body.
/// Used for exporting notes to .md files.
///
/// Format:
/// ```markdown
/// Title
/// #tag1 #tag2
///
/// Content body...
/// ```
pub fn note_to_markdown(note: &Note) -> String {
	let mut content = note.title.clone();

	if !note.tags.is_empty() {
		let tags_str = note.tags.iter().map(|t| format!("@{t}")).collect::<Vec<_>>().join(" ");
		content.push('\n');
		content.push_str(&tags_str);
	}

	if !note.content.is_empty() {
		content.push_str("\n\n");
		content.push_str(&note.content);
	}

	content
}

/// Resolves a note by ID or title pattern.
/// Returns the note ID if found, or an error if ambiguous/not found.
///
/// This function supports flexible note identification:
/// - Direct numeric ID: "42" -> finds note with ID 42
/// - Title pattern (case-insensitive): "groceries" -> finds notes containing
///   "groceries"
///
/// If multiple notes match a title pattern, returns an error with suggestions.
pub fn resolve_note(db: &Database, id_or_title: &str) -> Result<i64> {
	// Try parsing as ID first
	if let Ok(id) = id_or_title.parse::<i64>() {
		// Verify the ID exists
		if db.get_note(id)?.is_some() {
			return Ok(id);
		}
		anyhow::bail!("Note with ID {id} not found");
	}

	// Search by title pattern (case-insensitive)
	let all_notes = db.list_notes()?;
	let matches: Vec<Note> =
		all_notes.into_iter().filter(|n| n.title.to_lowercase().contains(&id_or_title.to_lowercase())).collect();

	match matches.len() {
		0 => anyhow::bail!("No notes found matching '{id_or_title}'"),
		1 => matches[0].id.ok_or_else(|| anyhow::anyhow!("Note missing ID")),
		_ => {
			eprintln!("Multiple notes found matching '{id_or_title}':");
			for note in &matches {
				if let Some(id) = note.id {
					let title = &note.title;
					eprintln!("  [{id}] {title}");
				}
			}
			anyhow::bail!("Please specify a more specific pattern or use the exact ID")
		}
	}
}
