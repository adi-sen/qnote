use anyhow::Result;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use ratatui::widgets::ListState;

use super::sorting::SortMode;
use crate::db::{Database, Note};

#[derive(Default)]
pub struct SearchState {
	pub query:         String,
	pub input_buffer:  String,
	pub match_indices: Vec<Vec<usize>>,
	matcher:           SkimMatcherV2,
}

impl SearchState {
	pub fn is_active(&self) -> bool { !self.query.is_empty() }

	pub fn clear(&mut self) {
		self.query.clear();
		self.input_buffer.clear();
		self.match_indices.clear();
	}

	pub fn set_query(&mut self, query: String) {
		self.query = query.clone();
		self.input_buffer = query;
	}

	pub fn refresh_notes(
		&mut self,
		db: &Database,
		sort_mode: SortMode,
		list_state: &mut ListState,
		preview_scroll: &mut u16,
	) -> Result<Vec<Note>> {
		let current_index = list_state.selected();
		let all_notes = db.list_notes()?;

		let notes = if self.query.is_empty() {
			self.match_indices.clear();
			let mut notes = all_notes;
			sort_mode.sort_notes(&mut notes);
			notes
		} else {
			let mut scored: Vec<_> = all_notes
				.into_iter()
				.filter_map(|note| {
					let text = format!("{} {}", note.title, note.content);
					self.matcher.fuzzy_indices(&text, &self.query).map(|(score, indices)| (note, score, indices))
				})
				.collect();

			scored.sort_unstable_by(|a, b| b.1.cmp(&a.1));
			let (notes, indices): (Vec<_>, Vec<_>) = scored.into_iter().map(|(note, _, indices)| (note, indices)).unzip();

			self.match_indices = indices;
			notes
		};

		list_state.select(if notes.is_empty() {
			None
		} else {
			Some(current_index.map_or(0, |idx| idx.min(notes.len() - 1)))
		});

		*preview_scroll = 0;
		Ok(notes)
	}
}
