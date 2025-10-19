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
use crate::utils::{
    format_date_full, format_date_only, note_to_markdown, parse_markdown_file, parse_tags,
    resolve_note, sanitize_filename,
};
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

/// Sort order for list command
#[derive(Clone, Copy, clap::ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum SortBy {
    Updated,
    Created,
    Title,
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
        #[arg(short, long)]
        tag: Option<String>,
        #[arg(short, long)]
        oneline: bool,
        #[arg(short, long, default_value = "updated")]
        sort: SortBy,
        #[arg(short, long)]
        limit: Option<usize>,
    },
    /// Show a specific note (by ID or title pattern)
    Show { id_or_title: String },
    /// Edit a note (by ID or title pattern)
    Edit {
        id_or_title: String,
        #[arg(short, long)]
        title: Option<String>,
        #[arg(short, long)]
        content: Option<String>,
        #[arg(short = 'g', long)]
        tags: Option<String>,
    },
    /// Delete a note (by ID or title pattern)
    Delete {
        id_or_title: String,
        #[arg(short, long)]
        yes: bool,
    },
    /// Search notes by keyword
    Search { query: String },
    /// Export a note to a markdown file
    Export {
        id_or_title: String,
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Import notes from markdown files
    Import { files: Vec<String> },
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
        } => cmd_add(db, title, content, tags),
        Commands::List {
            tag,
            oneline,
            sort,
            limit,
        } => cmd_list(db, tag, oneline, sort, limit),
        Commands::Show { id_or_title } => cmd_show(db, &id_or_title),
        Commands::Edit {
            id_or_title,
            title,
            content,
            tags,
        } => cmd_edit(db, &id_or_title, title, content, tags),
        Commands::Delete { id_or_title, yes } => cmd_delete(db, &id_or_title, yes),
        Commands::Search { query } => cmd_search(db, &query),
        Commands::Export {
            id_or_title,
            output,
        } => cmd_export(db, &id_or_title, output),
        Commands::Import { files } => cmd_import(db, &files),
        Commands::Tags => cmd_tags(db),
        Commands::Stats => cmd_stats(db),
        Commands::Tui => Ok(()), // Never reached - TUI is handled in main.rs
    }
}

fn cmd_add(db: &Database, title: String, content: String, tags: Option<String>) -> Result<()> {
    let tag_vec = parse_tags(tags);
    let note = Note::new(title, content, tag_vec);
    let id = db.create_note(&note)?;
    println!("Note created with ID: {id}");
    Ok(())
}

fn cmd_list(
    db: &Database,
    tag: Option<String>,
    oneline: bool,
    sort: SortBy,
    limit: Option<usize>,
) -> Result<()> {
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
    match sort {
        SortBy::Created => filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
        SortBy::Title => {
            filtered.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
        }
        SortBy::Updated => filtered.sort_by(|a, b| b.updated_at.cmp(&a.updated_at)),
    }

    // Limit
    if let Some(limit_val) = limit {
        filtered.truncate(limit_val);
    }

    if filtered.is_empty() {
        println!("No notes found.");
    } else if oneline {
        print_notes_oneline(&filtered);
    } else {
        print_notes_normal(&filtered);
    }
    Ok(())
}

fn print_notes_oneline(notes: &[Note]) {
    for note in notes {
        let tags_str = if note.tags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", note.tags.join(", "))
        };
        if let Some(id) = note.id {
            println!("{id}\t{}{tags_str}", note.title);
        }
    }
}

fn print_notes_normal(notes: &[Note]) {
    for note in notes {
        if let Some(id) = note.id {
            println!(
                "\n[{id}] {}\nTags: {}\nUpdated: {}",
                note.title,
                note.tags.join(", "),
                format_date_full(&note.updated_at)
            );
        }
    }
}

fn cmd_export(db: &Database, id_or_title: &str, output: Option<String>) -> Result<()> {
    let id = resolve_note(db, id_or_title)?;
    if let Some(note) = db.get_note(id)? {
        let filename = output.unwrap_or_else(|| format!("{}.md", sanitize_filename(&note.title)));
        let content = note_to_markdown(&note);

        std::fs::write(&filename, content)?;
        println!("Exported to: {filename}");
    }
    Ok(())
}

fn cmd_import(db: &Database, files: &[String]) -> Result<()> {
    use std::path::Path;

    let mut imported = 0;
    for file_path in files {
        let path = Path::new(file_path);
        if !path.exists() {
            eprintln!("Warning: File not found: {file_path}");
            continue;
        }

        let content = std::fs::read_to_string(path)?;

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

fn cmd_tags(db: &Database) -> Result<()> {
    let notes = db.list_notes()?;
    let mut tag_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

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

        let total = tags.len();
        println!("Tags ({total} total):");
        for (tag, count) in tags {
            println!("  {tag} ({count})");
        }
    }
    Ok(())
}

fn cmd_show(db: &Database, id_or_title: &str) -> Result<()> {
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

fn cmd_edit(
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

        db.update_note(id, new_title, new_content, &new_tags)?;
        println!("Note {id} updated.");
    }
    Ok(())
}

fn cmd_delete(db: &Database, id_or_title: &str, yes: bool) -> Result<()> {
    let id = resolve_note(db, id_or_title)?;
    if let Some(note) = db.get_note(id)? {
        eprintln!("Found: [{}] {}", id, note.title);
        if yes || confirm("Delete this note?") {
            db.delete_note(id)?;
            println!("Note {id} deleted.");
        } else {
            println!("Deletion cancelled.");
        }
    }
    Ok(())
}

fn cmd_search(db: &Database, query: &str) -> Result<()> {
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

fn cmd_stats(db: &Database) -> Result<()> {
    let notes = db.list_notes()?;
    if notes.is_empty() {
        println!("No notes yet!");
        return Ok(());
    }

    let (total_size, tag_set, oldest, newest) = notes.iter().fold(
        (0, std::collections::HashSet::new(), &notes[0], &notes[0]),
        |(size, mut tags, old, new), note| {
            tags.extend(note.tags.iter().cloned());
            (
                size + note.content.len() + note.title.len(),
                tags,
                if note.created_at < old.created_at {
                    note
                } else {
                    old
                },
                if note.updated_at > new.updated_at {
                    note
                } else {
                    new
                },
            )
        },
    );

    #[allow(
        clippy::cast_precision_loss,
        reason = "KB display doesn't require exact precision"
    )]
    let size_kb = total_size as f64 / 1024.0;
    let sep = "=".repeat(50);
    println!(
        "\n{sep}\nqnote Statistics\n{sep}\n\
        Total notes:      {}\n\
        Unique tags:      {}\n\
        Total size:       {size_kb:.2} KB\n\
        Oldest note:      {} ({})\n\
        Most recent:      {} ({})\n{sep}",
        notes.len(),
        tag_set.len(),
        oldest.title,
        format_date_only(&oldest.created_at),
        newest.title,
        format_date_full(&newest.updated_at)
    );
    Ok(())
}

/// Prompts user for confirmation. Returns true if user confirms.
fn confirm(prompt: &str) -> bool {
    use std::io::{self, Write};
    print!("{prompt} (y/N): ");
    let _ = io::stdout().flush();
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}
