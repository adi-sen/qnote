use anyhow::Result;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use super::{App, Screen, selection};
use crate::{db::Note, tui::editor::{open_editor_for_edit, open_editor_for_new_note}, utils::{note_to_markdown, sanitize_filename}};

impl App {
	#[allow(clippy::too_many_lines)]
	pub fn handle_list_input(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<bool> {
		if modifiers.contains(KeyModifiers::SHIFT) {
			return match key {
				KeyCode::Char('A') => {
					let count = self.selection.select_all(&self.notes);
					self.set_message(format!("Selected {count} notes"));
					Ok(false)
				}
				KeyCode::Char('C') => {
					let count = self.selection.clear();
					if count > 0 {
						self.set_message(format!("Cleared {count} selections"));
					}
					Ok(false)
				}
				KeyCode::Char('D') => {
					if self.selection.is_empty() {
						self.set_message("No notes selected");
					} else {
						let count = self.selection.delete_all(&self.db)?;
						self.set_message(format!("Deleted {count} notes"));
						self.refresh_notes()?;
					}
					Ok(false)
				}
				KeyCode::Char('X') => {
					if self.selection.is_empty() {
						self.set_message("No notes selected");
					} else {
						let (success, errors) = self.selection.export_all(&self.notes);
						self.set_message(if errors == 0 {
							format!("Exported {success} notes")
						} else {
							format!("Exported {success} notes ({errors} failed)")
						});
					}
					Ok(false)
				}
				_ => Ok(false),
			};
		}

		if modifiers.contains(KeyModifiers::CONTROL) {
			return match key {
				KeyCode::Char('c') => Ok(true),
				KeyCode::Char('j') => {
					self.scroll_preview(true);
					Ok(false)
				}
				KeyCode::Char('k') => {
					self.scroll_preview(false);
					Ok(false)
				}
				_ => Ok(false),
			};
		}

		match key {
			KeyCode::Char(' ') => {
				selection::toggle_and_navigate(
					&mut self.selection,
					&mut self.list_state,
					&self.notes,
					&mut self.preview_scroll,
				);
			}
			KeyCode::Char('.') => self.help_expanded = !self.help_expanded,
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
				self.set_message(format!("Sort: {}", self.sort_mode.name()));
			}
			KeyCode::Char(c) if c == self.config.keybindings.export => {
				if let Some(note) = self.get_selected_note() {
					let filename = format!("{}.md", sanitize_filename(&note.title));
					let msg = match std::fs::write(&filename, note_to_markdown(note)) {
						Ok(()) => format!("Exported to {filename}"),
						Err(e) => format!("Export failed: {e}"),
					};
					self.set_message(msg);
				}
			}
			KeyCode::Char(c) if c == self.config.keybindings.edit || key == KeyCode::Enter => {
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
				self.search.input_buffer = self.search.query.clone();
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
				let (had_search, had_selections) = (self.search.is_active(), !self.selection.is_empty());
				if had_search {
					self.search.clear();
					self.refresh_notes()?;
				}
				if had_selections {
					self.selection.clear();
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

	pub fn handle_search_input(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<bool> {
		if modifiers.contains(KeyModifiers::CONTROL)
			&& let Some(down) = match key {
				KeyCode::Char('n' | 'j') | KeyCode::Down => Some(true),
				KeyCode::Char('p' | 'k') | KeyCode::Up => Some(false),
				_ => None,
			} {
			self.navigate(down);
			return Ok(false);
		}

		match key {
			KeyCode::Esc => {
				self.search.input_buffer.clear();
				self.screen = Screen::List;
			}
			KeyCode::Enter => {
				self.screen = Screen::List;
				if self.search.is_active() {
					self.set_message(format!("Found {} notes", self.notes.len()));
				}
			}
			KeyCode::Backspace => {
				self.search.input_buffer.pop();
				self.search.set_query(self.search.input_buffer.clone());
				self.refresh_notes()?;
			}
			KeyCode::Down | KeyCode::Up => self.navigate(matches!(key, KeyCode::Down)),
			KeyCode::Char(c) => {
				self.search.input_buffer.push(c);
				self.search.set_query(self.search.input_buffer.clone());
				self.refresh_notes()?;
			}
			_ => {}
		}
		Ok(false)
	}
}
