//! Command-line interface for qnote.
//!
//! Provides CLI commands for managing notes without launching the TUI:
//! - add: Create a new note
//! - list: Display all notes or filter by tag
//! - show: View a specific note's details
//! - edit: Modify an existing note's title, content, or tags
//! - delete: Remove a note
//! - search: Find notes by keyword
//! - tui: Launch the interactive terminal UI (also the default if no command is given)

use crate::db::{Database, Note};
use anyhow::Result;
use clap::{Parser, Subcommand};

/// Main CLI structure parsed by clap.
#[derive(Parser)]
#[command(name = "qnote")]
#[command(about = "A quick note-taking app", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Available CLI commands.
#[derive(Subcommand)]
pub enum Commands {
    /// Add a new note with title, content, and optional tags
    Add {
        title: String,
        content: String,
        #[arg(short, long)]
        tags: Option<String>,
    },
    /// List all notes
    List {
        /// Filter by tag
        #[arg(short, long)]
        tag: Option<String>,
        /// Compact one-line format (good for piping to fzf)
        #[arg(short, long)]
        oneline: bool,
        /// Sort by: updated, created, title
        #[arg(short, long, default_value = "updated")]
        sort: String,
        /// Limit number of results
        #[arg(short, long)]
        limit: Option<usize>,
    },
    /// Show a specific note (by ID or title pattern)
    Show {
        /// Note ID or title pattern to search
        id_or_title: String,
    },
    /// Edit a note (by ID or title pattern)
    Edit {
        /// Note ID or title pattern to search
        id_or_title: String,
        /// New title
        #[arg(short, long)]
        title: Option<String>,
        /// New content
        #[arg(short, long)]
        content: Option<String>,
        /// New tags (comma-separated)
        #[arg(short = 'g', long)]
        tags: Option<String>,
    },
    /// Delete a note (by ID or title pattern)
    Delete {
        /// Note ID or title pattern to search
        id_or_title: String,
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
    /// Search notes
    Search {
        /// Search query
        query: String,
    },
    /// Export a note to a markdown file
    Export {
        /// Note ID or title pattern to search
        id_or_title: String,
        /// Output file path (defaults to <title>.md)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Import notes from markdown files
    Import {
        /// Markdown file paths
        files: Vec<String>,
    },
    /// List all tags with note counts
    Tags,
    /// Show statistics about notes
    Stats,
    /// Open TUI interface
    Tui,
}

/// Executes the given CLI command against the database.
/// Each command produces formatted output to stdout for user feedback.
pub fn handle_command(db: &Database, cmd: Commands) -> Result<()> {
    match cmd {
        Commands::Add {
            title,
            content,
            tags,
        } => {
            let tag_vec = parse_tags(tags);
            let note = Note::new(title, content, tag_vec);
            let id = db.create_note(&note)?;
            println!("Note created with ID: {}", id);
        }
        Commands::List {
            tag,
            oneline,
            sort,
            limit,
        } => {
            let notes = db.list_notes()?;

            // Filter by tag
            let mut filtered: Vec<Note> = if let Some(tag_filter) = tag {
                notes
                    .into_iter()
                    .filter(|n| n.tags.contains(&tag_filter))
                    .collect()
            } else {
                notes
            };

            // Sort
            match sort.as_str() {
                "created" => filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
                "title" => {
                    filtered.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
                }
                _ => filtered.sort_by(|a, b| b.updated_at.cmp(&a.updated_at)), // default "updated"
            }

            // Limit
            if let Some(limit_val) = limit {
                filtered.truncate(limit_val);
            }

            if filtered.is_empty() {
                println!("No notes found.");
            } else if oneline {
                // Compact format for piping to fzf or other tools
                for note in filtered {
                    let tags_str = if note.tags.is_empty() {
                        String::new()
                    } else {
                        format!(" [{}]", note.tags.join(", "))
                    };
                    println!("{}\t{}{}", note.id.unwrap(), note.title, tags_str);
                }
            } else {
                // Normal format
                for note in filtered {
                    println!("\n[{}] {}", note.id.unwrap(), note.title);
                    println!("Tags: {}", note.tags.join(", "));
                    println!("Updated: {}", note.updated_at.format("%Y-%m-%d %H:%M"));
                }
            }
        }
        Commands::Show { id_or_title } => {
            let id = resolve_note(db, &id_or_title)?;
            if let Some(note) = db.get_note(id)? {
                println!("\n{}", "=".repeat(50));
                println!("Title: {}", note.title);
                println!("Tags: {}", note.tags.join(", "));
                println!("Created: {}", note.created_at.format("%Y-%m-%d %H:%M"));
                println!("Updated: {}", note.updated_at.format("%Y-%m-%d %H:%M"));
                println!("{}", "=".repeat(50));
                println!("\n{}", note.content);
                println!();
            }
        }
        Commands::Edit {
            id_or_title,
            title,
            content,
            tags,
        } => {
            let id = resolve_note(db, &id_or_title)?;
            if let Some(note) = db.get_note(id)? {
                let new_title = title.unwrap_or(note.title);
                let new_content = content.unwrap_or(note.content);
                let new_tags = tags.map(|t| parse_tags(Some(t))).unwrap_or(note.tags);

                db.update_note(id, new_title, new_content, new_tags)?;
                println!("Note {} updated.", id);
            }
        }
        Commands::Delete { id_or_title, yes } => {
            let id = resolve_note(db, &id_or_title)?;
            let note = db.get_note(id)?;

            if let Some(note) = note {
                // Show what will be deleted
                eprintln!("Found: [{}] {}", id, note.title);

                // Ask for confirmation unless --yes flag is provided
                if yes || confirm("Delete this note?") {
                    db.delete_note(id)?;
                    println!("Note {} deleted.", id);
                } else {
                    println!("Deletion cancelled.");
                }
            }
        }
        Commands::Search { query } => {
            let notes = db.search_notes(&query)?;
            if notes.is_empty() {
                println!("No notes found matching '{}'.", query);
            } else {
                println!("Found {} note(s):", notes.len());
                for note in notes {
                    println!("\n[{}] {}", note.id.unwrap(), note.title);
                    println!("Tags: {}", note.tags.join(", "));
                }
            }
        }
        Commands::Export {
            id_or_title,
            output,
        } => {
            let id = resolve_note(db, &id_or_title)?;
            if let Some(note) = db.get_note(id)? {
                // Determine output filename
                let filename = output.unwrap_or_else(|| {
                    format!("{}.md", note.title.replace('/', "-").replace(' ', "_"))
                });

                // Format note as markdown
                let mut content = note.title.clone();
                if !note.tags.is_empty() {
                    content.push_str(&format!(
                        "\n{}",
                        note.tags
                            .iter()
                            .map(|t| format!("#{}", t))
                            .collect::<Vec<String>>()
                            .join(" ")
                    ));
                }
                if !note.content.is_empty() {
                    content.push_str(&format!("\n\n{}", note.content));
                }

                std::fs::write(&filename, content)?;
                println!("Exported to: {}", filename);
            }
        }
        Commands::Import { files } => {
            use std::path::Path;

            let mut imported = 0;
            for file_path in files {
                let path = Path::new(&file_path);
                if !path.exists() {
                    eprintln!("Warning: File not found: {}", file_path);
                    continue;
                }

                let content = std::fs::read_to_string(path)?;

                // Parse markdown file
                if let Some((title, note_content, tags)) = parse_markdown_file(&content) {
                    let note = Note::new(title, note_content, tags);
                    db.create_note(&note)?;
                    imported += 1;
                    println!("Imported: {}", path.display());
                } else {
                    eprintln!("Warning: Could not parse: {}", file_path);
                }
            }
            println!("\nImported {} note(s)", imported);
        }
        Commands::Tags => {
            let notes = db.list_notes()?;
            let mut tag_counts: std::collections::HashMap<String, usize> =
                std::collections::HashMap::new();

            for note in notes {
                for tag in note.tags {
                    *tag_counts.entry(tag).or_insert(0) += 1;
                }
            }

            if tag_counts.is_empty() {
                println!("No tags found.");
            } else {
                let mut tags: Vec<(String, usize)> = tag_counts.into_iter().collect();
                tags.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

                println!("Tags ({} total):", tags.len());
                for (tag, count) in tags {
                    println!("  {} ({})", tag, count);
                }
            }
        }
        Commands::Stats => {
            let notes = db.list_notes()?;

            if notes.is_empty() {
                println!("No notes yet!");
                return Ok(());
            }

            let total = notes.len();
            let mut total_size = 0;
            let mut tag_set = std::collections::HashSet::new();
            let mut oldest = &notes[0];
            let mut newest = &notes[0];

            for note in &notes {
                total_size += note.content.len() + note.title.len();
                for tag in &note.tags {
                    tag_set.insert(tag.clone());
                }
                if note.created_at < oldest.created_at {
                    oldest = note;
                }
                if note.updated_at > newest.updated_at {
                    newest = note;
                }
            }

            println!("\n{}", "=".repeat(50));
            println!("qnote Statistics");
            println!("{}", "=".repeat(50));
            println!("Total notes:      {}", total);
            println!("Unique tags:      {}", tag_set.len());
            println!("Total size:       {:.2} KB", total_size as f64 / 1024.0);
            println!(
                "Oldest note:      {} ({})",
                oldest.title,
                oldest.created_at.format("%Y-%m-%d")
            );
            println!(
                "Most recent:      {} ({})",
                newest.title,
                newest.updated_at.format("%Y-%m-%d %H:%M")
            );
            println!("{}", "=".repeat(50));
        }
        Commands::Tui => {
            // This branch is never reached - TUI command is handled in main.rs before calling this function
            unreachable!()
        }
    }
    Ok(())
}

/// Parses a comma-separated string of tags into a vector.
/// Trims whitespace and filters out empty strings.
///
/// Example: "work, important, todo" -> vec!["work", "important", "todo"]
fn parse_tags(tags: Option<String>) -> Vec<String> {
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
fn resolve_note(db: &Database, id_or_title: &str) -> Result<i64> {
    // Try parsing as ID first
    if let Ok(id) = id_or_title.parse::<i64>() {
        // Verify the ID exists
        if db.get_note(id)?.is_some() {
            return Ok(id);
        } else {
            anyhow::bail!("Note with ID {} not found", id);
        }
    }

    // Search by title pattern (case-insensitive)
    let all_notes = db.list_notes()?;
    let matches: Vec<Note> = all_notes
        .into_iter()
        .filter(|n| n.title.to_lowercase().contains(&id_or_title.to_lowercase()))
        .collect();

    match matches.len() {
        0 => anyhow::bail!("No notes found matching '{}'", id_or_title),
        1 => Ok(matches[0].id.unwrap()),
        _ => {
            eprintln!("Multiple notes found matching '{}':", id_or_title);
            for note in &matches {
                eprintln!("  [{}] {}", note.id.unwrap(), note.title);
            }
            anyhow::bail!("Please specify a more specific pattern or use the exact ID")
        }
    }
}

/// Prompts user for confirmation. Returns true if user confirms.
fn confirm(prompt: &str) -> bool {
    use std::io::{self, Write};

    print!("{} (y/N): ", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

/// Parses a markdown file into (title, content, tags).
/// Expected format:
/// Line 1: Title
/// Line 2 (optional): #tag1 #tag2
/// Remaining: Content
fn parse_markdown_file(content: &str) -> Option<(String, String, Vec<String>)> {
    let content = content.trim();
    if content.is_empty() {
        return None;
    }

    let mut lines = content.lines();
    let title = lines.next()?.trim().to_string();

    if title.is_empty() {
        return None;
    }

    let mut tags = Vec::new();
    let mut note_content = Vec::new();
    let mut found_tags = false;

    for line in lines {
        let trimmed = line.trim();

        // Check if line 2 contains tags
        if !found_tags && !trimmed.is_empty() {
            if trimmed.starts_with('#') {
                // Parse tags
                tags = trimmed
                    .split_whitespace()
                    .filter(|word| word.starts_with('#'))
                    .map(|tag| tag.trim_start_matches('#').to_string())
                    .filter(|tag| !tag.is_empty())
                    .collect();
                found_tags = true;
                continue;
            } else {
                note_content.push(line);
                found_tags = true;
            }
        } else if found_tags || !trimmed.is_empty() {
            note_content.push(line);
            found_tags = true;
        }
    }

    let final_content = note_content.join("\n").trim().to_string();
    Some((title, final_content, tags))
}
