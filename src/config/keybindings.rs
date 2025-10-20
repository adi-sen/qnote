use serde::{Deserialize, Serialize};

/// Keybindings configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingsConfig {
	/// Key to quit the application
	#[serde(default = "default_quit_key")]
	pub quit: char,

	/// Key to create a new note
	#[serde(default = "default_new_note_key")]
	pub new_note: char,

	/// Key to delete a note
	#[serde(default = "default_delete_key")]
	pub delete: char,

	/// Key to edit a note
	#[serde(default = "default_edit_key")]
	pub edit: char,

	/// Key to start search
	#[serde(default = "default_search_key")]
	pub search: char,

	/// Key to export note
	#[serde(default = "default_export_key")]
	pub export: char,

	/// Key to cycle sort mode
	#[serde(default = "default_sort_key")]
	pub sort: char,

	/// Key to go to top
	#[serde(default = "default_goto_top_key")]
	pub goto_top: char,

	/// Key to go to bottom
	#[serde(default = "default_goto_bottom_key")]
	pub goto_bottom: char,

	/// Key to move down
	#[serde(default = "default_move_down_key")]
	pub move_down: char,

	/// Key to move up
	#[serde(default = "default_move_up_key")]
	pub move_up: char,
}

const fn default_quit_key() -> char { 'q' }

const fn default_new_note_key() -> char { 'n' }

const fn default_delete_key() -> char { 'd' }

const fn default_edit_key() -> char { 'e' }

const fn default_search_key() -> char { '/' }

const fn default_export_key() -> char { 'x' }

const fn default_sort_key() -> char { 's' }

const fn default_goto_top_key() -> char { 'g' }

const fn default_goto_bottom_key() -> char { 'G' }

const fn default_move_down_key() -> char { 'j' }

const fn default_move_up_key() -> char { 'k' }

impl Default for KeybindingsConfig {
	fn default() -> Self {
		Self {
			quit:        default_quit_key(),
			new_note:    default_new_note_key(),
			delete:      default_delete_key(),
			edit:        default_edit_key(),
			search:      default_search_key(),
			export:      default_export_key(),
			sort:        default_sort_key(),
			goto_top:    default_goto_top_key(),
			goto_bottom: default_goto_bottom_key(),
			move_down:   default_move_down_key(),
			move_up:     default_move_up_key(),
		}
	}
}
