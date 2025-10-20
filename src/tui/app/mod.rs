mod input;
mod navigation;
mod search;
mod selection;
mod sorting;

use anyhow::Result;
use ratatui::widgets::ListState;
pub use search::SearchState;
pub use selection::SelectionState;
pub use sorting::SortMode;

use crate::{config::Config, db::{Database, Note}};

#[derive(PartialEq, Eq)]
pub enum Screen {
	List,
	SearchMode,
}

pub struct App {
	pub db:             Database,
	pub config:         Config,
	pub screen:         Screen,
	pub notes:          Vec<Note>,
	pub list_state:     ListState,
	pub message:        Option<String>,
	pub needs_clear:    bool,
	pub preview_scroll: u16,
	pub sort_mode:      SortMode,
	pub help_expanded:  bool,
	pub search:         SearchState,
	pub selection:      SelectionState,
	message_counter:    u8,
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
			message: None,
			message_counter: 0,
			needs_clear: false,
			preview_scroll: 0,
			sort_mode: SortMode::UpdatedDesc,
			help_expanded: false,
			search: SearchState::default(),
			selection: SelectionState::default(),
		})
	}

	pub fn set_message(&mut self, msg: impl Into<String>) {
		self.message = Some(msg.into());
		self.message_counter = self.config.ui.message_display_keypresses;
	}

	pub fn tick_message(&mut self) {
		self.message_counter = self.message_counter.saturating_sub(1);
		if self.message_counter == 0 {
			self.message = None;
		}
	}

	pub fn get_selected_note(&self) -> Option<&Note> { self.list_state.selected().and_then(|i| self.notes.get(i)) }

	pub fn is_note_selected(&self, note_id: i64) -> bool { self.selection.contains(note_id) }

	fn refresh_notes(&mut self) -> Result<()> {
		self.notes = self.search.refresh_notes(&self.db, self.sort_mode, &mut self.list_state, &mut self.preview_scroll)?;
		Ok(())
	}

	fn navigate(&mut self, down: bool) {
		selection::navigate_list(&mut self.list_state, &self.notes, &mut self.preview_scroll, down);
	}

	fn scroll_preview(&mut self, down: bool) {
		if let Some(note) = self.get_selected_note() {
			let content_height = navigation::get_preview_content_height(note, &self.config.ui);
			navigation::scroll_preview(&mut self.preview_scroll, down, content_height, &self.config.ui);
		}
	}
}
