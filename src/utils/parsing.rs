//! Parsing utilities for markdown and tags.

/// Extracts @tags from text and returns (cleaned_text, tags)
fn extract_tags(text: &str) -> (String, Vec<String>) {
	let mut result = String::with_capacity(text.len());
	let mut tags = Vec::new();
	let mut chars = text.chars().peekable();

	while let Some(ch) = chars.next() {
		if ch == '@' {
			// Check if next char is alphanumeric or underscore
			if chars.peek().is_some_and(|c| c.is_alphanumeric() || *c == '_') {
				let mut tag = String::new();
				while let Some(&c) = chars.peek() {
					if c.is_alphanumeric() || c == '_' {
						tag.push(c);
						chars.next();
					} else {
						break;
					}
				}
				if !tag.is_empty() {
					tags.push(tag);
				}
			} else {
				result.push(ch);
			}
		} else {
			result.push(ch);
		}
	}

	(result, tags)
}

/// Parses a markdown file according to qnote's format.
///
/// Expected format:
/// - Line 1: Title (required, but can be empty - will use fallback)
/// - Remaining lines: Note content (body)
/// - Tags can appear anywhere in content using @tag format
///
/// Returns None if the note is completely empty.
/// Returns Some((title, content, tags)) for a valid note.
/// If no explicit title is provided, generates one from the content.
/// Tags are extracted from entire content and removed from final content.
pub fn parse_markdown_file(content: &str) -> Option<(String, String, Vec<String>)> {
	let content = content.trim();
	if content.is_empty() {
		return None;
	}

	let mut lines = content.lines();
	let mut title = lines.next()?.trim().to_string();
	let remaining_content = lines.collect::<Vec<_>>().join("\n");

	// Extract @tags from content
	let (note_content, tags) = extract_tags(&remaining_content);
	let note_content = note_content.trim().to_string();

	// Title fallback: if no title, generate from content
	if title.is_empty() {
		if note_content.is_empty() {
			return None;
		}
		// Extract first word, stripping markdown headers if present
		title = note_content
			.lines()
			.next()
			.and_then(|first_line| {
				let cleaned = first_line.trim_start_matches('#').trim_start_matches(' ').trim();
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
	tags.map(|t| t.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()).unwrap_or_default()
}
