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
        Commands::List { tag, oneline } => {
            let notes = db.list_notes()?;
            let filtered: Vec<_> = if let Some(tag_filter) = tag {
                notes
                    .into_iter()
                    .filter(|n| n.tags.contains(&tag_filter))
                    .collect()
            } else {
                notes
            };

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
    let matches: Vec<_> = all_notes
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
