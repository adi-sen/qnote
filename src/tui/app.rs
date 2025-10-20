use std::collections::HashSet;

use anyhow::Result;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use ratatui::{crossterm::event::{KeyCode, KeyModifiers}, widgets::ListState};

use super::editor::{open_editor_for_edit, open_editor_for_new_note};
use crate::{config::Config, db::{Database, Note}, utils::{note_to_markdown, sanitize_filename}};

/// Current UI screen mode.
#[derive(PartialEq, Eq)]
pub enum Screen {
	List,
	SearchMode,
}

/// Note sorting mode (cycle with 's' key).
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum SortMode {
	UpdatedDesc,
	UpdatedAsc,
	TitleAsc,
	TitleDesc,
	CreatedDesc,
	CreatedAsc,
}

impl SortMode {
	/// Returns the next sort mode in the cycle.
	pub const fn next(self) -> Self {
		match self {
			Self::UpdatedDesc => Self::UpdatedAsc,
			Self::UpdatedAsc => Self::TitleAsc,
			Self::TitleAsc => Self::TitleDesc,
			Self::TitleDesc => Self::CreatedDesc,
			Self::CreatedDesc => Self::CreatedAsc,
			Self::CreatedAsc => Self::UpdatedDesc,
		}
	}

	/// Returns the display name for the UI.
	pub const fn name(self) -> &'static str {
		match self {
			Self::UpdatedDesc => "Updated ↓",
			Self::UpdatedAsc => "Updated ↑",
			Self::TitleAsc => "Title A→Z",
			Self::TitleDesc => "Title Z→A",
			Self::CreatedDesc => "Created ↓",
			Self::CreatedAsc => "Created ↑",
		}
	}
}

/// TUI application state.
pub struct App {
	pub db:              Database,
	pub config:          Config,
	pub screen:          Screen,
	pub notes:           Vec<Note>,
	pub list_state:      ListState,
	pub input_buffer:    String,
	pub search_query:    String,
	pub message:         Option<String>,
	pub message_counter: u8,
	pub needs_clear:     bool,
	pub preview_scroll:  u16,
	pub sort_mode:       SortMode,
	pub match_indices:   Vec<Vec<usize>>,
	pub selected_notes:  HashSet<i64>,
	pub help_expanded:   bool,
	fuzzy_matcher:       SkimMatcherV2,
}
impl App {
	pub fn new(db: Database, config: Config) -> Result<Self> {
		let notes = db.list_notes()?;
		let mut list_state = ListState::default();
		if !notes.is_empty() {
			list_state.select(Some(0));
		}

		Ok(Self {
			db,
			config,
			screen: Screen::List,
			notes,
			list_state,
			input_buffer: String::new(),
			search_query: String::new(),
			message: None,
			message_counter: 0,
			needs_clear: false,
			preview_scroll: 0,
			sort_mode: SortMode::UpdatedDesc,
			match_indices: Vec::new(),
			selected_notes: HashSet::new(),
			help_expanded: false,
			fuzzy_matcher: SkimMatcherV2::default(),
		})
	}

	/// Sets a status message that auto-clears after a few keypresses.
	fn set_message(&mut self, msg: impl Into<String>) {
		self.message = Some(msg.into());
		self.message_counter = self.config.ui.message_display_keypresses;
	}

	/// Decrements message counter and clears when it reaches zero.
	pub fn tick_message(&mut self) {
		self.message_counter = self.message_counter.saturating_sub(1);
		if self.message_counter == 0 {
			self.message = None;
		}
	}

	/// Refreshes notes from database, applying fuzzy search and sort.
	/// Preserves cursor position where possible.
	fn refresh_notes(&mut self) -> Result<()> {
		let current_index = self.list_state.selected();
		let all_notes = self.db.list_notes()?;

		if self.search_query.is_empty() {
			self.notes = all_notes;
			self.match_indices.clear();
			self.sort_notes();
		} else {
			let mut scored_notes: Vec<(Note, i64, Vec<usize>)> = Vec::with_capacity(all_notes.len());

			for note in all_notes {
				let search_text = format!("{} {}", note.title, note.content);

				if let Some((score, indices)) = self.fuzzy_matcher.fuzzy_indices(&search_text, &self.search_query) {
					scored_notes.push((note, score, indices));
				}
			}
			scored_notes.sort_unstable_by(|a, b| b.1.cmp(&a.1));

			let (notes_vec, indices_vec): (Vec<Note>, Vec<Vec<usize>>) =
				scored_notes.into_iter().map(|(note, _score, indices)| (note, indices)).unzip();

			self.notes = notes_vec;
			self.match_indices = indices_vec;
		}

		// Preserve cursor position, clamping to valid range
		let new_index = if self.notes.is_empty() {
			None
		} else if let Some(idx) = current_index {
			Some(idx.min(self.notes.len() - 1))
		} else {
			Some(0)
		};

		self.list_state.select(new_index);
		self.preview_scroll = 0;
		Ok(())
	}

	/// Sorts the notes list in-place according to the current sort mode.
	fn sort_notes(&mut self) {
		self.notes.sort_unstable_by(|a, b| match self.sort_mode {
			SortMode::UpdatedDesc => b.updated_at.cmp(&a.updated_at),
			SortMode::UpdatedAsc => a.updated_at.cmp(&b.updated_at),
			SortMode::TitleAsc => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
			SortMode::TitleDesc => b.title.to_lowercase().cmp(&a.title.to_lowercase()),
			SortMode::CreatedDesc => b.created_at.cmp(&a.created_at),
			SortMode::CreatedAsc => a.created_at.cmp(&b.created_at),
		});
	}

	/// Returns a reference to the currently selected note, if any.
	pub fn get_selected_note(&self) -> Option<&Note> { self.list_state.selected().and_then(|i| self.notes.get(i)) }

	/// Checks if a note is currently selected (for multi-select).
	pub fn is_note_selected(&self, note_id: i64) -> bool { self.selected_notes.contains(&note_id) }

	/// Toggles selection of the currently highlighted note and navigates down.
	fn toggle_current_selection(&mut self) {
		if let Some(note) = self.get_selected_note()
			&& let Some(id) = note.id
		{
			if self.selected_notes.contains(&id) {
				self.selected_notes.remove(&id);
			} else {
				self.selected_notes.insert(id);
			}
			// Navigate down after toggling (yazi-style behavior)
			self.navigate(true);
		}
	}

	/// Selects all currently visible notes.
	fn select_all_notes(&mut self) {
		for note in &self.notes {
			if let Some(id) = note.id {
				self.selected_notes.insert(id);
			}
		}
		let count = self.selected_notes.len();
		self.set_message(format!("Selected {count} notes"));
	}

	/// Clears all note selections.
	fn clear_all_selections(&mut self) {
		let count = self.selected_notes.len();
		self.selected_notes.clear();
		if count > 0 {
			self.set_message(format!("Cleared {count} selections"));
		}
	}

	/// Deletes all selected notes from the database.
	fn delete_selected_notes(&mut self) -> Result<()> {
		if self.selected_notes.is_empty() {
			self.set_message("No notes selected");
			return Ok(());
		}

		let count = self.selected_notes.len();
		for note_id in self.selected_notes.drain() {
			self.db.delete_note(note_id)?;
		}
		self.set_message(format!("Deleted {count} notes"));
		self.refresh_notes()?;
		Ok(())
	}

	/// Exports all selected notes to markdown files.
	fn export_selected_notes(&mut self) -> Result<()> {
		if self.selected_notes.is_empty() {
			self.set_message("No notes selected");
			return Ok(());
		}

		let mut success_count = 0;
		let mut error_count = 0;

		for note in &self.notes {
			if let Some(id) = note.id
				&& self.selected_notes.contains(&id)
			{
				let filename = format!("{}.md", sanitize_filename(&note.title));
				let content = note_to_markdown(note);

				match std::fs::write(&filename, &content) {
					Ok(()) => success_count += 1,
					Err(_) => error_count += 1,
				}
			}
		}

		let msg = if error_count == 0 {
			format!("Exported {success_count} notes")
		} else {
			format!("Exported {success_count} notes ({error_count} failed)")
		};
		self.set_message(msg);
		self.selected_notes.clear();
		Ok(())
	}

	/// Navigate through notes list (without wrapping)
	fn navigate(&mut self, down: bool) {
		if !self.notes.is_empty() {
			let current = self.list_state.selected().unwrap_or(0);
			let new_index = if down { (current + 1).min(self.notes.len() - 1) } else { current.saturating_sub(1) };
			self.list_state.select(Some(new_index));
			self.preview_scroll = 0;
		}
	}

	/// Estimates the height of the preview content for scroll bounds checking.
	fn get_preview_content_height(&self) -> u16 {
		self.get_selected_note().map_or(0, |note| {
			#[allow(clippy::cast_possible_truncation)]
			let lines = note.content.lines().count() as u16;
			#[allow(clippy::cast_possible_truncation)]
			let headers = (note.content.matches('#').count() as u16).min(self.config.ui.max_markdown_formatting_buffer);

			self.config.ui.header_lines + lines + headers
		})
	}

	/// Scroll the preview pane up or down
	fn scroll_preview(&mut self, down: bool) {
		if down {
			let content_height = self.get_preview_content_height();
			let max_scroll = content_height.saturating_sub(self.config.ui.preview_max_scroll_buffer);
			self.preview_scroll = (self.preview_scroll + self.config.ui.preview_scroll_step).min(max_scroll);
		} else {
			self.preview_scroll = self.preview_scroll.saturating_sub(self.config.ui.preview_scroll_step);
		}
	}

	/// Handles keyboard input in list view mode.
	#[allow(clippy::too_many_lines)]
	pub fn handle_list_input(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<bool> {
		if modifiers.contains(KeyModifiers::SHIFT) {
			match key {
				KeyCode::Char('A') => {
					self.select_all_notes();
					return Ok(false);
				}
				KeyCode::Char('C') => {
					self.clear_all_selections();
					return Ok(false);
				}
				KeyCode::Char('D') => {
					self.delete_selected_notes()?;
					return Ok(false);
				}
				KeyCode::Char('X') => {
					self.export_selected_notes()?;
					return Ok(false);
				}
				_ => {}
			}
		}

		if modifiers.contains(KeyModifiers::CONTROL) {
			match key {
				KeyCode::Char('c') => return Ok(true), // Ctrl+C to quit
				KeyCode::Char('j') => {
					self.scroll_preview(true);
					return Ok(false);
				}
				KeyCode::Char('k') => {
					self.scroll_preview(false);
					return Ok(false);
				}
				_ => {}
			}
		}

		match key {
			KeyCode::Char(' ') => {
				self.toggle_current_selection();
			}
			KeyCode::Char('.') => {
				self.help_expanded = !self.help_expanded;
			}
			KeyCode::Char(c) if c == self.config.keybindings.quit => return Ok(true),
			KeyCode::Char(c) if c == self.config.keybindings.new_note || c == 'a' => {
				let msg = match open_editor_for_new_note(&self.config.editor) {
					Ok(Some((title, content, tags))) => {
						self.db.create_note(&Note::new(title, content, tags))?;
						self.refresh_notes()?;
						"Note created"
					}
					_ => "Cancelled",
				};
				self.set_message(msg);
				self.needs_clear = true;
			}
			KeyCode::Char(c) if c == self.config.keybindings.delete => {
				if let Some(note) = self.get_selected_note()
					&& let Some(id) = note.id
				{
					let title = &note.title;
					self.db.delete_note(id)?;
					self.set_message(format!("Deleted '{title}'"));
					self.refresh_notes()?;
				}
			}
			KeyCode::Char(c) if c == self.config.keybindings.sort => {
				self.sort_mode = self.sort_mode.next();
				self.refresh_notes()?;
				let sort_name = self.sort_mode.name();
				self.set_message(format!("Sort: {sort_name}"));
			}
			KeyCode::Char(c) if c == self.config.keybindings.export => {
				if let Some(note) = self.get_selected_note() {
					let filename = format!("{}.md", sanitize_filename(&note.title));
					let content = note_to_markdown(note);

					let msg = match std::fs::write(&filename, &content) {
						Ok(()) => format!("Exported to {filename}"),
						Err(e) => format!("Export failed: {e}"),
					};
					self.set_message(msg);
				}
			}
			KeyCode::Char(c) if c == self.config.keybindings.edit => {
				if let Some(note) = self.get_selected_note().cloned()
					&& let Some(id) = note.id
				{
					match open_editor_for_edit(&note, &self.config.editor) {
						Ok(Some((title, content, tags))) => {
							self.db.update_note(id, &title, &content, &tags)?;
							self.set_message("Note saved");
							self.refresh_notes()?;
						}
						_ => self.set_message("Cancelled"),
					}
					self.needs_clear = true;
				}
			}
			KeyCode::Enter => {
				if let Some(note) = self.get_selected_note().cloned()
					&& let Some(id) = note.id
				{
					match open_editor_for_edit(&note, &self.config.editor) {
						Ok(Some((title, content, tags))) => {
							self.db.update_note(id, &title, &content, &tags)?;
							self.set_message("Note saved");
							self.refresh_notes()?;
						}
						_ => self.set_message("Cancelled"),
					}
					self.needs_clear = true;
				}
			}
			KeyCode::Char(c) if c == self.config.keybindings.search => {
				self.screen = Screen::SearchMode;
				self.input_buffer = self.search_query.clone();
			}
			KeyCode::Char(c) if c == self.config.keybindings.goto_top => {
				if !self.notes.is_empty() {
					self.list_state.select(Some(0));
					self.preview_scroll = 0;
				}
			}
			KeyCode::Char(c) if c == self.config.keybindings.goto_bottom => {
				if !self.notes.is_empty() {
					self.list_state.select(Some(self.notes.len() - 1));
					self.preview_scroll = 0;
				}
			}
			KeyCode::Down => self.navigate(true),
			KeyCode::Up => self.navigate(false),
			KeyCode::Char(c) if c == self.config.keybindings.move_down => self.navigate(true),
			KeyCode::Char(c) if c == self.config.keybindings.move_up => self.navigate(false),
			KeyCode::Esc => {
				let had_search = !self.search_query.is_empty();
				let had_selections = !self.selected_notes.is_empty();

				if had_search {
					self.search_query.clear();
					self.input_buffer.clear();
					self.refresh_notes()?;
				}

				if had_selections {
					self.selected_notes.clear();
				}

				if had_search && had_selections {
					self.set_message("Cleared search and selections");
				} else if had_search {
					self.set_message("Search cleared");
				} else if had_selections {
					self.set_message("Selections cleared");
				}
			}
			_ => {}
		}
		Ok(false)
	}

	/// Handles keyboard input in search mode (incremental fuzzy search).
	pub fn handle_search_input(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<bool> {
		if modifiers.contains(KeyModifiers::CONTROL) {
			let navigate = match key {
				KeyCode::Char('n' | 'j') | KeyCode::Down => Some(true),
				KeyCode::Char('p' | 'k') | KeyCode::Up => Some(false),
				_ => None,
			};

			if let Some(down) = navigate {
				self.navigate(down);
				return Ok(false);
			}
		}

		match key {
			KeyCode::Esc => {
				self.input_buffer.clear();
				self.screen = Screen::List;
			}
			KeyCode::Enter => {
				self.screen = Screen::List;
				if !self.search_query.is_empty() {
					let count = self.notes.len();
					self.set_message(format!("Found {count} notes"));
				}
			}
			KeyCode::Backspace => {
				self.input_buffer.pop();
				self.search_query = self.input_buffer.clone();
				self.refresh_notes()?;
			}
			KeyCode::Down | KeyCode::Up => {
				self.navigate(matches!(key, KeyCode::Down));
			}
			KeyCode::Char(c) => {
				self.input_buffer.push(c);
				self.search_query = self.input_buffer.clone();
				self.refresh_notes()?;
			}
			_ => {}
		}
		Ok(false)
	}
}
