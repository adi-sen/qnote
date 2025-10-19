use crate::db::{Database, Note};
use crate::utils::{note_to_markdown, sanitize_filename};
use anyhow::Result;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::widgets::ListState;

use super::editor::{open_editor_for_edit, open_editor_for_new_note};

// Constants for UI behavior
const MESSAGE_DISPLAY_KEYPRESSES: u8 = 5;
const PREVIEW_SCROLL_STEP: u16 = 3;
const PREVIEW_MAX_SCROLL_BUFFER: u16 = 10;
const HEADER_LINES: u16 = 3; // title + metadata + blank line
const MAX_MARKDOWN_FORMATTING_BUFFER: u16 = 10;

/// Application screen state: determines which UI mode is active.
#[derive(PartialEq, Eq)]
pub enum Screen {
    /// Normal list view where user can navigate and edit notes
    List,
    /// Inline search mode with incremental fuzzy filtering
    SearchMode,
}

/// Note sorting options that user can cycle through with 's' key.
#[derive(PartialEq, Eq, Clone, Copy)]
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

    /// Returns the human-readable name for displaying in the UI footer.
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
    /// Cached fuzzy matcher to avoid recreating on every keystroke
    fuzzy_matcher: SkimMatcherV2,
}
impl App {
    pub fn new(db: Database) -> Result<Self> {
        let notes = db.list_notes()?;
        let mut list_state = ListState::default();
        if !notes.is_empty() {
            list_state.select(Some(0));
        }

        Ok(Self {
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
            fuzzy_matcher: SkimMatcherV2::default(),
        })
    }

    /// Sets a status message that will auto-clear after a few keypresses.
    fn set_message(&mut self, msg: impl Into<String>) {
        self.message = Some(msg.into());
        self.message_counter = MESSAGE_DISPLAY_KEYPRESSES;
    }

    /// Decrements the message counter on each keypress and clears the message when it reaches zero.
    pub fn tick_message(&mut self) {
        self.message_counter = self.message_counter.saturating_sub(1);
        if self.message_counter == 0 {
            self.message = None;
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
        if self.search_query.is_empty() {
            self.notes = notes;
            self.match_indices.clear();

            // Apply sort mode
            self.sort_notes();
        } else {
            // Pre-allocate with capacity hint to reduce reallocations
            let mut scored_notes: Vec<(Note, i64, Vec<usize>)> = Vec::with_capacity(notes.len());

            for note in notes {
                // Build search text once, reusing tag string when possible
                let search_text = if note.tags.is_empty() {
                    format!("{} {}", note.title, note.content)
                } else {
                    format!("{} {} {}", note.title, note.content, note.tags.join(" "))
                };

                // Use cached fuzzy matcher instead of creating new one
                if let Some((score, indices)) = self
                    .fuzzy_matcher
                    .fuzzy_indices(&search_text, &self.search_query)
                {
                    scored_notes.push((note, score, indices));
                }
            }

            // Sort by fuzzy match score (higher is better)
            scored_notes.sort_unstable_by(|a, b| b.1.cmp(&a.1));

            // Unpack into separate vectors (more idiomatic than explicit loop)
            let (notes_vec, indices_vec): (Vec<Note>, Vec<Vec<usize>>) = scored_notes
                .into_iter()
                .map(|(note, _score, indices)| (note, indices))
                .unzip();

            self.notes = notes_vec;
            self.match_indices = indices_vec;
        }

        self.list_state
            .select((!self.notes.is_empty()).then_some(0));
        self.preview_scroll = 0; // Reset scroll when notes change
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
    pub fn get_selected_note(&self) -> Option<&Note> {
        self.list_state.selected().and_then(|i| self.notes.get(i))
    }

    /// Navigate through notes list (with wrapping)
    fn navigate(&mut self, down: bool) {
        if !self.notes.is_empty() {
            let i = self.list_state.selected().map_or(0, |i| {
                if down {
                    if i >= self.notes.len() - 1 { 0 } else { i + 1 }
                } else if i == 0 {
                    self.notes.len() - 1
                } else {
                    i - 1
                }
            });
            self.list_state.select(Some(i));
            self.preview_scroll = 0;
        }
    }

    /// Estimates the height of the preview content for scroll bounds checking.
    fn get_preview_content_height(&self) -> u16 {
        self.get_selected_note().map_or(0, |note| {
            #[allow(clippy::cast_possible_truncation)]
            let lines = note.content.lines().count() as u16;
            #[allow(clippy::cast_possible_truncation)]
            let headers =
                (note.content.matches('#').count() as u16).min(MAX_MARKDOWN_FORMATTING_BUFFER);
            HEADER_LINES + lines + headers
        })
    }

    /// Scroll the preview pane up or down
    fn scroll_preview(&mut self, down: bool) {
        if down {
            let content_height = self.get_preview_content_height();
            let max_scroll = content_height.saturating_sub(PREVIEW_MAX_SCROLL_BUFFER);
            self.preview_scroll = (self.preview_scroll + PREVIEW_SCROLL_STEP).min(max_scroll);
        } else {
            self.preview_scroll = self.preview_scroll.saturating_sub(PREVIEW_SCROLL_STEP);
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
    #[allow(clippy::too_many_lines)]
    pub fn handle_list_input(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<bool> {
        // Handle Ctrl+j/k for preview scrolling
        if modifiers.contains(KeyModifiers::CONTROL) {
            match key {
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
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('n' | 'a') => {
                let msg = match open_editor_for_new_note() {
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
            KeyCode::Char('d') => {
                if let Some(note) = self.get_selected_note()
                    && let Some(id) = note.id
                {
                    let title = &note.title;
                    self.db.delete_note(id)?;
                    self.set_message(format!("Deleted '{title}'"));
                    self.refresh_notes()?;
                }
            }
            KeyCode::Char('s') => {
                self.sort_mode = self.sort_mode.next();
                self.refresh_notes()?;
                let sort_name = self.sort_mode.name();
                self.set_message(format!("Sort: {sort_name}"));
            }
            KeyCode::Char('x') => {
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
            KeyCode::Char('e') | KeyCode::Enter => {
                if let Some(note) = self.get_selected_note().cloned()
                    && let Some(id) = note.id
                {
                    match open_editor_for_edit(&note) {
                        Ok(Some((title, content, tags))) => {
                            self.db.update_note(id, title, content, &tags)?;
                            self.set_message("Note saved");
                            self.refresh_notes()?;
                        }
                        _ => self.set_message("Cancelled"),
                    }
                    self.needs_clear = true;
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
            KeyCode::Down | KeyCode::Char('j') => self.navigate(true),
            KeyCode::Up | KeyCode::Char('k') => self.navigate(false),
            KeyCode::Esc => {
                if !self.search_query.is_empty() {
                    self.search_query.clear();
                    self.input_buffer.clear();
                    self.refresh_notes()?;
                    self.set_message("Filter cleared");
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
