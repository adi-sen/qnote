use crate::db::Note;
use anyhow::{Context, Result};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    env, fs,
    io::{self, Write},
    path::PathBuf,
    process::Command,
};

/// Returns the user's preferred editor from environment variables.
/// Checks $EDITOR, falling back to vi if not set.
fn get_editor() -> String {
    env::var("EDITOR").unwrap_or_else(|_| "vi".to_string())
}

/// Returns a consistent temp file path for qnote editing.
/// Reusing the same path avoids filesystem overhead and is faster than creating new files.
fn get_temp_path() -> PathBuf {
    env::temp_dir().join("qnote-edit.md")
}

/// Opens the user's editor with an empty template for creating a new note.
/// Returns None if the user cancels or creates an empty note.
/// Returns Some((title, content, tags)) if a valid note is created.
pub fn open_editor_for_new_note() -> Result<Option<(String, String, Vec<String>)>> {
    let temp_path = get_temp_path();

    // Use write! instead of fs::write for consistency
    let mut file = fs::File::create(&temp_path)?;
    // Empty template - just create empty file
    file.flush()?;
    drop(file);

    open_editor(&temp_path)?;

    // Read and parse the edited content
    let content = fs::read_to_string(&temp_path)?;

    parse_note_file(&content)
}

/// Opens the user's editor with an existing note's content pre-filled.
/// Note format: title on line 1, hashtags on line 2 (if any), content after blank line.
/// Returns None if the user cancels or deletes all content.
/// Returns Some((title, content, tags)) if the note is successfully edited.
pub fn open_editor_for_edit(note: &Note) -> Result<Option<(String, String, Vec<String>)>> {
    let temp_path = get_temp_path();

    // Use BufWriter for better I/O performance
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
            write!(writer, "#{}", tag)?;
        }
    }

    // Write content if present
    if !note.content.is_empty() {
        writer.write_all(b"\n\n")?;
        write!(writer, "{}", note.content)?;
    }

    writer.flush()?;
    drop(writer);

    open_editor(&temp_path)?;

    // Read back and parse the edited content
    let content = fs::read_to_string(&temp_path)?;

    parse_note_file(&content)
}

/// Opens the user's preferred editor for the given file path.
/// Temporarily exits the TUI alternate screen and raw mode, then restores them after editing.
/// This allows the editor to function normally without interference from the TUI.
fn open_editor(path: &std::path::Path) -> Result<()> {
    let editor = get_editor();

    // Temporarily exit TUI mode so editor can take over the terminal
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    let status = Command::new(&editor)
        .arg(path)
        .status()
        .context(format!("Failed to open editor: {}", editor))?;

    // Restore TUI mode after editor closes
    execute!(io::stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;

    if !status.success() {
        anyhow::bail!("Editor exited with non-zero status");
    }

    Ok(())
}

/// Parses a note file according to qnote's format:
/// - Line 1: Title (required)
/// - Line 2: Hashtags (optional, must start with #)
/// - Remaining lines: Note content (body)
///
/// Returns None if the note is empty or invalid.
/// Returns Some((title, content, tags)) for a valid note.
fn parse_note_file(content: &str) -> Result<Option<(String, String, Vec<String>)>> {
    let content = content.trim();
    if content.is_empty() {
        return Ok(None);
    }

    let mut lines = content.lines();
    let mut title = String::new();
    let mut tags = Vec::new();
    let mut note_content = Vec::new();

    // Line 1: Extract title
    if let Some(first_line) = lines.next() {
        title = first_line.trim().to_string();
    }

    // Line 2: Check for hashtags, otherwise it's content
    let mut started_content = false;
    for line in lines {
        if !started_content {
            let trimmed = line.trim();

            if trimmed.starts_with('#') {
                // Parse hashtags: split on whitespace and extract tag names
                tags = trimmed
                    .split_whitespace()
                    .filter(|word| word.starts_with('#'))
                    .map(|tag| tag.trim_start_matches('#').to_string())
                    .filter(|tag| !tag.is_empty())
                    .collect();
                started_content = true;
                continue;
            } else if !trimmed.is_empty() {
                // Not a tag line, so it's the start of content
                note_content.push(line);
                started_content = true;
            }
            // Skip empty lines between title and content/tags
        } else {
            // Collect all remaining lines as content
            note_content.push(line);
        }
    }

    let note_content = note_content.join("\n").trim().to_string();

    // Title fallback: if no title, use first word of content
    if title.is_empty() {
        if !note_content.is_empty() {
            // Extract first word, stripping markdown headers if present
            let first_line = note_content.lines().next().unwrap_or("");
            let cleaned = first_line
                .trim_start_matches("# ")
                .trim_start_matches("## ")
                .trim_start_matches("### ")
                .trim();

            title = cleaned
                .split_whitespace()
                .next()
                .unwrap_or("Untitled")
                .to_string();
        } else {
            // Empty note: no title and no content, cancel creation
            return Ok(None);
        }
    }

    Ok(Some((title, note_content, tags)))
}
