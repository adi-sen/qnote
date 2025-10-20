use std::collections::HashSet;

use anyhow::Result;

use crate::{cli::SortBy, db::{Database, Note}, utils::{format_date_full, format_date_only}};

/// Handles the list command - displays all notes with optional filtering
pub fn handle_list(
	db: &Database,
	tag: Option<String>,
	oneline: bool,
	sort: SortBy,
	limit: Option<usize>,
) -> Result<()> {
	let notes = db.list_notes()?;

	let mut filtered: Vec<Note> = if let Some(tag_filter) = tag {
		notes.into_iter().filter(|n| n.tags.contains(&tag_filter)).collect()
	} else {
		notes
	};

	match sort {
		SortBy::Created => filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
		SortBy::Title => {
			filtered.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
		}
		SortBy::Updated => filtered.sort_by(|a, b| b.updated_at.cmp(&a.updated_at)),
	}

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

/// Handles the tags command - lists all tags with note counts
pub fn handle_tags(db: &Database) -> Result<()> {
	let notes = db.list_notes()?;

	// Pre-allocate HashMap capacity
	let estimated_capacity = notes.iter().map(|n| n.tags.len()).sum();
	let mut tag_counts: std::collections::HashMap<String, usize> =
		std::collections::HashMap::with_capacity(estimated_capacity);

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

/// Handles the stats command - shows note statistics
pub fn handle_stats(db: &Database) -> Result<()> {
	let notes = db.list_notes()?;
	if notes.is_empty() {
		println!("No notes yet!");
		return Ok(());
	}

	let (total_size, tag_set, oldest, newest) =
		notes.iter().fold((0, HashSet::new(), &notes[0], &notes[0]), |(size, mut tags, old, new), note| {
			tags.extend(note.tags.iter().cloned());
			(
				size + note.content.len() + note.title.len(),
				tags,
				if note.created_at < old.created_at { note } else { old },
				if note.updated_at > new.updated_at { note } else { new },
			)
		});

	let size_kb = total_size as f64 / 1024.0;
	let sep = "=".repeat(50);
	println!(
		"\n{sep}\nqnote Statistics\n{sep}\n\
        Total notes:      {}\n\
        Unique tags:      {}\n\
        Total size:       {:.2} KB\n\
        Oldest note:      {} ({})\n\
        Most recent:      {} ({})\n{sep}",
		notes.len(),
		tag_set.len(),
		size_kb,
		oldest.title,
		format_date_only(&oldest.created_at),
		newest.title,
		format_date_full(&newest.updated_at)
	);
	Ok(())
}

fn print_notes_oneline(notes: &[Note]) {
	for note in notes {
		let tags_str = if note.tags.is_empty() { String::new() } else { format!(" [{}]", note.tags.join(", ")) };
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
