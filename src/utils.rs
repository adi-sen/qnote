//! Shared utility functions used across CLI and TUI modules.

use crate::db::{Database, Note};
use anyhow::Result;
use chrono::{DateTime, Utc};

/// Sanitizes a note title for use as a filename.
/// Replaces '/' with '-' and spaces with '_' to create filesystem-safe names.
///
/// # Examples
/// ```
/// assert_eq!(sanitize_filename("My Note"), "My_Note");
/// assert_eq!(sanitize_filename("Path/To/Note"), "Path-To-Note");
/// ```
pub fn sanitize_filename(title: &str) -> String {
    title.replace('/', "-").replace(' ', "_")
}

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
        let tags_str = note
            .tags
            .iter()
            .map(|t| format!("#{t}"))
            .collect::<Vec<_>>()
            .join(" ");
        content.push('\n');
        content.push_str(&tags_str);
    }

    if !note.content.is_empty() {
        content.push_str("\n\n");
        content.push_str(&note.content);
    }

    content
}

/// Parses a markdown file according to qnote's format.
///
/// Expected format:
/// - Line 1: Title (required, but can be empty - will use fallback)
/// - Line 2: Hashtags (optional, must start with #)
/// - Remaining lines: Note content (body)
///
/// Returns None if the note is completely empty.
/// Returns Some((title, content, tags)) for a valid note.
/// If no explicit title is provided, generates one from the content.
pub fn parse_markdown_file(content: &str) -> Option<(String, String, Vec<String>)> {
    let content = content.trim();
    if content.is_empty() {
        return None;
    }

    let mut lines = content.lines();
    let mut title = lines.next()?.trim().to_string();

    // Check if second line has tags, otherwise it's content
    let second_line = lines.next().unwrap_or("");
    let (tags, rest): (Vec<String>, Vec<&str>) = if second_line.trim().starts_with('#') {
        let tags = second_line
            .split_whitespace()
            .filter_map(|word| word.strip_prefix('#'))
            .filter(|tag| !tag.is_empty())
            .map(ToString::to_string)
            .collect();
        (tags, lines.collect())
    } else {
        (
            Vec::new(),
            std::iter::once(second_line).chain(lines).collect(),
        )
    };

    let note_content = rest.join("\n").trim().to_string();

    // Title fallback: if no title, generate from content
    if title.is_empty() {
        if note_content.is_empty() {
            // Empty note: no title and no content, cancel creation
            return None;
        }
        // Extract first word, stripping markdown headers if present
        title = note_content
            .lines()
            .next()
            .and_then(|first_line| {
                let cleaned = first_line
                    .trim_start_matches('#')
                    .trim_start_matches(' ')
                    .trim();
                cleaned.split_whitespace().next()
            })
            .unwrap_or("Untitled")
            .to_string();
    }

    Some((title, note_content, tags))
}

/// Parses a comma-separated string of tags into a vector.
/// Trims whitespace and filters out empty strings.
pub fn parse_tags(tags: Option<String>) -> Vec<String> {
    tags.map(|t| {
        t.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    })
    .unwrap_or_default()
}

/// Resolves a note by ID or title pattern.
/// Returns the note ID if found, or an error if ambiguous/not found.
///
/// This function supports flexible note identification:
/// - Direct numeric ID: "42" -> finds note with ID 42
/// - Title pattern (case-insensitive): "groceries" -> finds notes containing "groceries"
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
    let matches: Vec<Note> = all_notes
        .into_iter()
        .filter(|n| n.title.to_lowercase().contains(&id_or_title.to_lowercase()))
        .collect();

    match matches.len() {
        0 => anyhow::bail!("No notes found matching '{id_or_title}'"),
        1 => matches[0]
            .id
            .ok_or_else(|| anyhow::anyhow!("Note missing ID")),
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

/// Date format constants for consistent formatting across the application.
pub mod date_formats {
    /// Short format for list views: "Jan 15"
    pub const SHORT: &str = "%b %d";

    /// Full format for detailed views: "2024-01-15 14:30"
    pub const FULL: &str = "%Y-%m-%d %H:%M";

    /// Date only format: "2024-01-15"
    pub const DATE_ONLY: &str = "%Y-%m-%d";
}

/// Formats a datetime for list view display (short format).
/// Returns: "Jan 15"
pub fn format_date_short(dt: &DateTime<Utc>) -> String {
    dt.format(date_formats::SHORT).to_string()
}

/// Formats a datetime for detailed view display (full format).
/// Returns: "2024-01-15 14:30"
pub fn format_date_full(dt: &DateTime<Utc>) -> String {
    dt.format(date_formats::FULL).to_string()
}

/// Formats a datetime as date only (no time).
/// Returns: "2024-01-15"
pub fn format_date_only(dt: &DateTime<Utc>) -> String {
    dt.format(date_formats::DATE_ONLY).to_string()
}
