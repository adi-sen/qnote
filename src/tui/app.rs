use crate::db::{Database, Note};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use ratatui::widgets::ListState;

use super::editor::{open_editor_for_edit, open_editor_for_new_note};

/// Application screen state: determines which UI mode is active.
#[derive(PartialEq)]
pub enum Screen {
    /// Normal list view where user can navigate and edit notes
    List,
    /// Inline search mode with incremental fuzzy filtering
    SearchMode,
}

/// Note sorting options that user can cycle through with 's' key.
#[derive(PartialEq, Clone, Copy)]
pub enum SortMode {
    /// Sort by last updated date, newest first (default)
    UpdatedDesc,
    /// Sort by last updated date, oldest first
    UpdatedAsc,
    /// Sort by title alphabetically, A→Z
    TitleAsc,
    /// Sort by title alphabetically, Z→A
    TitleDesc,
    /// Sort by creation date, newest first
    CreatedDesc,
    /// Sort by creation date, oldest first
    CreatedAsc,
}

impl SortMode {
    /// Cycles to the next sort mode when user presses 's' key.
    pub fn next(self) -> Self {
        match self {
            SortMode::UpdatedDesc => SortMode::UpdatedAsc,
            SortMode::UpdatedAsc => SortMode::TitleAsc,
            SortMode::TitleAsc => SortMode::TitleDesc,
            SortMode::TitleDesc => SortMode::CreatedDesc,
            SortMode::CreatedDesc => SortMode::CreatedAsc,
            SortMode::CreatedAsc => SortMode::UpdatedDesc,
        }
    }

    /// Returns the human-readable name for displaying in the UI footer.
    pub fn name(self) -> &'static str {
        match self {
            SortMode::UpdatedDesc => "Updated ↓",
            SortMode::UpdatedAsc => "Updated ↑",
            SortMode::TitleAsc => "Title A→Z",
            SortMode::TitleDesc => "Title Z→A",
            SortMode::CreatedDesc => "Created ↓",
            SortMode::CreatedAsc => "Created ↑",
        }
    }
}

/// Main application state for the TUI.
pub struct App {
    pub db: Database,
    pub screen: Screen,
    pub notes: Vec<Note>,
    pub list_state: ListState,
    pub input_buffer: String,
    pub search_query: String,
    pub message: Option<String>,
    pub message_counter: u8,
    pub needs_clear: bool,
    pub preview_scroll: u16,
    pub sort_mode: SortMode,
    pub match_indices: Vec<Vec<usize>>,
}
impl App {
    pub fn new(db: Database) -> Result<Self> {
        let notes = db.list_notes()?;
        let mut list_state = ListState::default();
        if !notes.is_empty() {
            list_state.select(Some(0));
        }

        Ok(App {
            db,
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
        })
    }

    /// Sets a status message that will auto-clear after 5 keypresses.
    /// Used for feedback like "Note saved", "Deleted 'Shopping'", etc.
    fn set_message(&mut self, msg: String) {
        self.message = Some(msg);
        self.message_counter = 5;
    }

    /// Decrements the message counter on each keypress and clears the message when it reaches zero.
    /// Called automatically in the main event loop.
    pub fn tick_message(&mut self) {
        if self.message_counter > 0 {
            self.message_counter -= 1;
            if self.message_counter == 0 {
                self.message = None;
            }
        }
    }

    /// Refreshes the notes list from the database, applying search and sort filters.
    ///
    /// If a search query exists, performs fuzzy matching across title, content, and tags,
    /// then sorts by relevance score. Otherwise applies the current sort mode.
    /// Also resets selection and scroll state as needed.
    fn refresh_notes(&mut self) -> Result<()> {
        let notes = self.db.list_notes()?;

        // Apply fuzzy search if query exists
        if !self.search_query.is_empty() {
            let matcher = SkimMatcherV2::default();
            let mut scored_notes: Vec<(Note, i64, Vec<usize>)> = notes
                .into_iter()
                .filter_map(|note| {
                    // Search in title, content, and tags
                    let search_text =
                        format!("{} {} {}", note.title, note.content, note.tags.join(" "));

                    matcher
                        .fuzzy_indices(&search_text, &self.search_query)
                        .map(|(score, indices)| (note, score, indices))
                })
                .collect();

            // Sort by fuzzy match score (higher is better)
            scored_notes.sort_by(|a, b| b.1.cmp(&a.1));

            self.notes = scored_notes
                .iter()
                .map(|(note, _, _)| note.clone())
                .collect();
            self.match_indices = scored_notes
                .iter()
                .map(|(_, _, indices)| indices.clone())
                .collect();
        } else {
            self.notes = notes;
            self.match_indices = Vec::new();

            // Apply sort mode
            self.sort_notes();
        }

        if !self.notes.is_empty() && self.list_state.selected().is_none() {
            self.list_state.select(Some(0));
        } else if self.notes.is_empty() {
            self.list_state.select(None);
        }
        self.preview_scroll = 0; // Reset scroll when notes change
        Ok(())
    }

    /// Sorts the notes list in-place according to the current sort mode.
    /// Only called when no search query is active (search results are sorted by relevance).
    fn sort_notes(&mut self) {
        match self.sort_mode {
            SortMode::UpdatedDesc => {
                self.notes.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
            }
            SortMode::UpdatedAsc => {
                self.notes.sort_by(|a, b| a.updated_at.cmp(&b.updated_at));
            }
            SortMode::TitleAsc => {
                self.notes
                    .sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
            }
            SortMode::TitleDesc => {
                self.notes
                    .sort_by(|a, b| b.title.to_lowercase().cmp(&a.title.to_lowercase()));
            }
            SortMode::CreatedDesc => {
                self.notes.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            }
            SortMode::CreatedAsc => {
                self.notes.sort_by(|a, b| a.created_at.cmp(&b.created_at));
            }
        }
    }

    /// Returns a reference to the currently selected note, if any.
    pub fn get_selected_note(&self) -> Option<&Note> {
        self.list_state.selected().and_then(|i| self.notes.get(i))
    }

    /// Estimates the height of the preview content for scroll bounds checking.
    /// Used to prevent scrolling beyond the actual content length.
    fn get_preview_content_height(&self) -> u16 {
        if let Some(note) = self.get_selected_note() {
            // Count header lines (title, metadata, blank line) plus content lines
            let mut count = 3u16;
            count += note.content.lines().count() as u16;
            // Add buffer for markdown formatting (headers, lists, etc.)
            count += (note.content.matches('#').count() as u16).min(10);
            count
        } else {
            0
        }
    }

    /// Handles keyboard input when in normal list view mode.
    /// Returns true if the application should quit, false otherwise.
    ///
    /// Key bindings:
    /// - ^j/k: Scroll preview pane
    /// - j/k: Navigate notes list
    /// - g/G: Jump to top/bottom
    /// - n: New note
    /// - e/Enter: Edit note
    /// - d: Delete note
    /// - s: Cycle sort modes
    /// - x: Export note
    /// - /: Enter search mode
    /// - ESC: Clear search filter
    /// - q: Quit
    pub fn handle_list_input(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<bool> {
        // Handle Ctrl+j/k for preview scrolling
        if modifiers.contains(KeyModifiers::CONTROL) {
            match key {
                KeyCode::Char('j') => {
                    // Scroll preview down (with bounds checking)
                    let content_height = self.get_preview_content_height();
                    let max_scroll = content_height.saturating_sub(10);
                    self.preview_scroll = (self.preview_scroll + 3).min(max_scroll);
                    return Ok(false);
                }
                KeyCode::Char('k') => {
                    // Scroll preview up
                    self.preview_scroll = self.preview_scroll.saturating_sub(3);
                    return Ok(false);
                }
                _ => {}
            }
        }

        match key {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('n') | KeyCode::Char('a') => {
                if let Ok(Some((title, content, tags))) = open_editor_for_new_note() {
                    let note = Note::new(title, content, tags);
                    self.db.create_note(&note)?;
                    self.set_message("Note created".to_string());
                    self.refresh_notes()?;
                    self.needs_clear = true;
                } else {
                    self.set_message("Cancelled".to_string());
                    self.needs_clear = true;
                }
            }
            KeyCode::Char('d') => {
                if let Some(note) = self.get_selected_note() {
                    let title = note.title.clone();
                    let id = note.id.unwrap();
                    self.db.delete_note(id)?;
                    self.set_message(format!("Deleted '{}'", title));
                    self.refresh_notes()?;
                }
            }
            KeyCode::Char('s') => {
                // Cycle through sort modes
                self.sort_mode = self.sort_mode.next();
                self.refresh_notes()?;
                self.set_message(format!("Sort: {}", self.sort_mode.name()));
            }
            KeyCode::Char('x') => {
                // Export current note
                if let Some(note) = self.get_selected_note() {
                    let filename = format!("{}.md", note.title.replace("/", "-"));
                    let mut content = note.title.clone();
                    if !note.tags.is_empty() {
                        content.push_str(&format!(
                            "\n{}",
                            note.tags
                                .iter()
                                .map(|t| format!("#{}", t))
                                .collect::<Vec<_>>()
                                .join(" ")
                        ));
                    }
                    if !note.content.is_empty() {
                        content.push_str(&format!("\n\n{}", note.content));
                    }

                    match std::fs::write(&filename, content) {
                        Ok(_) => self.set_message(format!("Exported to {}", filename)),
                        Err(e) => self.set_message(format!("Export failed: {}", e)),
                    }
                }
            }
            KeyCode::Char('e') => {
                if let Some(note) = self.get_selected_note() {
                    let note_clone = note.clone();
                    let id = note_clone.id.unwrap();
                    if let Ok(Some((title, content, tags))) = open_editor_for_edit(&note_clone) {
                        self.db.update_note(id, title, content, tags)?;
                        self.set_message("Note saved".to_string());
                        self.refresh_notes()?;
                        self.needs_clear = true;
                    } else {
                        self.set_message("Cancelled".to_string());
                        self.needs_clear = true;
                    }
                }
            }
            KeyCode::Char('/') => {
                self.screen = Screen::SearchMode;
                self.input_buffer = self.search_query.clone(); // Start with current search
            }
            KeyCode::Char('g') => {
                if !self.notes.is_empty() {
                    self.list_state.select(Some(0));
                    self.preview_scroll = 0;
                }
            }
            KeyCode::Char('G') => {
                if !self.notes.is_empty() {
                    self.list_state.select(Some(self.notes.len() - 1));
                    self.preview_scroll = 0;
                }
            }
            KeyCode::Enter => {
                if let Some(note) = self.get_selected_note() {
                    let note_clone = note.clone();
                    let id = note_clone.id.unwrap();
                    if let Ok(Some((title, content, tags))) = open_editor_for_edit(&note_clone) {
                        self.db.update_note(id, title, content, tags)?;
                        self.set_message("Note saved".to_string());
                        self.refresh_notes()?;
                        self.needs_clear = true;
                    } else {
                        self.set_message("Cancelled".to_string());
                        self.needs_clear = true;
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.notes.is_empty() {
                    let i = match self.list_state.selected() {
                        Some(i) => {
                            if i >= self.notes.len() - 1 {
                                0
                            } else {
                                i + 1
                            }
                        }
                        None => 0,
                    };
                    self.list_state.select(Some(i));
                    self.preview_scroll = 0;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.notes.is_empty() {
                    let i = match self.list_state.selected() {
                        Some(i) => {
                            if i == 0 {
                                self.notes.len() - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    self.list_state.select(Some(i));
                    self.preview_scroll = 0;
                }
            }
            KeyCode::Esc => {
                // Clear search filter
                if !self.search_query.is_empty() {
                    self.search_query.clear();
                    self.input_buffer.clear();
                    self.refresh_notes()?;
                    self.set_message("Filter cleared".to_string());
                }
            }
            _ => {}
        }
        Ok(false)
    }

    /// Handles keyboard input when in search mode (incremental fuzzy search).
    /// Returns true if the application should quit, false otherwise.
    ///
    /// Key bindings:
    /// - Type: Add to search query (incremental filtering)
    /// - Backspace: Remove from search query
    /// - ^n/p or ^j/k: Navigate results while typing
    /// - Up/Down: Navigate results
    /// - Enter: Accept search and return to list
    /// - ESC: Cancel search
    pub fn handle_search_input(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<bool> {
        // Handle Ctrl+n/p or Ctrl+j/k for navigation while searching
        if modifiers.contains(KeyModifiers::CONTROL) {
            match key {
                KeyCode::Char('n') | KeyCode::Char('j') | KeyCode::Down => {
                    // Navigate down
                    if !self.notes.is_empty() {
                        let i = match self.list_state.selected() {
                            Some(i) => {
                                if i >= self.notes.len() - 1 {
                                    0
                                } else {
                                    i + 1
                                }
                            }
                            None => 0,
                        };
                        self.list_state.select(Some(i));
                        self.preview_scroll = 0;
                    }
                    return Ok(false);
                }
                KeyCode::Char('p') | KeyCode::Char('k') | KeyCode::Up => {
                    // Navigate up
                    if !self.notes.is_empty() {
                        let i = match self.list_state.selected() {
                            Some(i) => {
                                if i == 0 {
                                    self.notes.len() - 1
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                        self.list_state.select(Some(i));
                        self.preview_scroll = 0;
                    }
                    return Ok(false);
                }
                _ => {}
            }
        }

        match key {
            KeyCode::Esc => {
                // Cancel search - restore previous query
                self.input_buffer.clear();
                self.screen = Screen::List;
            }
            KeyCode::Enter => {
                // Accept search and exit search mode
                self.screen = Screen::List;
                if !self.search_query.is_empty() {
                    self.set_message(format!("Found {} notes", self.notes.len()));
                }
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
                // Incremental search - update results as user deletes
                self.search_query = self.input_buffer.clone();
                self.refresh_notes()?;
            }
            KeyCode::Down => {
                // Arrow keys also work for navigation
                if !self.notes.is_empty() {
                    let i = match self.list_state.selected() {
                        Some(i) => {
                            if i >= self.notes.len() - 1 {
                                0
                            } else {
                                i + 1
                            }
                        }
                        None => 0,
                    };
                    self.list_state.select(Some(i));
                    self.preview_scroll = 0;
                }
            }
            KeyCode::Up => {
                // Arrow keys also work for navigation
                if !self.notes.is_empty() {
                    let i = match self.list_state.selected() {
                        Some(i) => {
                            if i == 0 {
                                self.notes.len() - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    self.list_state.select(Some(i));
                    self.preview_scroll = 0;
                }
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
                // Incremental search - update results as user types
                self.search_query = self.input_buffer.clone();
                self.refresh_notes()?;
            }
            _ => {}
        }
        Ok(false)
    }
}
