use std::collections::HashSet;

use anyhow::Result;
use ratatui::widgets::ListState;

use crate::{db::{Database, Note}, utils::{note_to_markdown, sanitize_filename}};

#[derive(Default)]
pub struct SelectionState {
	pub selected_notes: HashSet<i64>,
}

impl SelectionState {
	pub fn contains(&self, note_id: i64) -> bool { self.selected_notes.contains(&note_id) }

	pub fn toggle(&mut self, note_id: i64) {
		if !self.selected_notes.remove(&note_id) {
			self.selected_notes.insert(note_id);
		}
	}

	pub fn select_all(&mut self, notes: &[Note]) -> usize {
		self.selected_notes.extend(notes.iter().filter_map(|n| n.id));
		self.selected_notes.len()
	}

	pub fn clear(&mut self) -> usize {
		let count = self.len();
		self.selected_notes.clear();
		count
	}

	pub fn is_empty(&self) -> bool { self.selected_notes.is_empty() }

	pub fn len(&self) -> usize { self.selected_notes.len() }

	pub fn delete_all(&mut self, db: &Database) -> Result<usize> {
		let count = self.len();
		for note_id in self.selected_notes.drain() {
			db.delete_note(note_id)?;
		}
		Ok(count)
	}

	pub fn export_all(&mut self, notes: &[Note]) -> (usize, usize) {
		let (success, errors) =
			notes.iter().filter(|n| n.id.is_some_and(|id| self.selected_notes.contains(&id))).fold((0, 0), |(s, e), note| {
				let filename = format!("{}.md", sanitize_filename(&note.title));
				match std::fs::write(&filename, note_to_markdown(note)) {
					Ok(()) => (s + 1, e),
					Err(_) => (s, e + 1),
				}
			});

		self.selected_notes.clear();
		(success, errors)
	}
}

pub fn toggle_and_navigate(
	selection: &mut SelectionState,
	list_state: &mut ListState,
	notes: &[Note],
	preview_scroll: &mut u16,
) {
	if let Some(i) = list_state.selected()
		&& let Some(note) = notes.get(i)
		&& let Some(id) = note.id
	{
		selection.toggle(id);
		navigate_list(list_state, notes, preview_scroll, true);
	}
}

pub fn navigate_list(list_state: &mut ListState, notes: &[Note], preview_scroll: &mut u16, down: bool) {
	if !notes.is_empty() {
		let current = list_state.selected().unwrap_or(0);
		let new_index = if down { (current + 1).min(notes.len() - 1) } else { current.saturating_sub(1) };
		list_state.select(Some(new_index));
		*preview_scroll = 0;
	}
}
