//! Parsing utilities for markdown and tags.

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
		(Vec::new(), std::iter::once(second_line).chain(lines).collect())
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
