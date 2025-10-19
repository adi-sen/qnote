use super::app::{App, Screen};
use super::markdown::markdown_to_lines;
use crate::utils::format_date_short;
use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyEventKind};
use ratatui::{
    Terminal,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

// UI layout constants
const LIST_WIDTH_PERCENT: u16 = 40;
const PREVIEW_WIDTH_PERCENT: u16 = 60;
const LIST_BORDER_PADDING: u16 = 4;

const CYAN_BOLD: Style = Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD);
const CYAN: Style = Style::new().fg(Color::Cyan);
const DARK_GRAY: Style = Style::new().fg(Color::DarkGray);
const GRAY: Style = Style::new().fg(Color::Gray);

const HELP_LIST_MODE: &str = "j/k navigate  ^j/k scroll  g/G top/bottom  ⏎ edit  n new  e edit  d delete  s sort  x export  / search  ESC clear  q quit";
const HELP_SEARCH_MODE: &str = "^n/p navigate  ⏎ accept  ESC cancel";

pub fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        // Only clear if returning from editor
        if app.needs_clear {
            terminal.clear()?;
            app.needs_clear = false;
        }
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            app.tick_message(); // Auto-clear messages after keypresses

            let should_quit = match app.screen {
                Screen::List => app.handle_list_input(key.code, key.modifiers)?,
                Screen::SearchMode => app.handle_search_input(key.code, key.modifiers)?,
            };

            if should_quit {
                return Ok(());
            }
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &mut App) {
    // Layout: main content, optional status bar, footer
    let has_message = app.message.is_some();
    let footer_height = calculate_footer_height(app, f.area().width);

    let constraints = if has_message {
        vec![
            Constraint::Min(0),
            Constraint::Length(1), // Status bar
            Constraint::Length(footer_height),
        ]
    } else {
        vec![Constraint::Min(0), Constraint::Length(footer_height)]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(f.area());

    // Always render split view - search is inline now
    render_split_view(f, app, chunks[0]);

    if has_message {
        render_status_bar(f, app, chunks[1]);
        render_help(f, app, chunks[2]);
    } else {
        render_help(f, app, chunks[1]);
    }
}

const fn calculate_footer_height(app: &App, width: u16) -> u16 {
    let help_text = match app.screen {
        Screen::List => HELP_LIST_MODE,
        Screen::SearchMode => HELP_SEARCH_MODE,
    };

    // Calculate if text needs wrapping
    if help_text.len() > width as usize {
        2
    } else {
        1
    }
}

fn render_status_bar(f: &mut ratatui::Frame, app: &App, area: Rect) {
    if let Some(msg) = &app.message {
        let status = Paragraph::new(msg.as_str()).style(Style::default().fg(Color::Yellow));
        f.render_widget(status, area);
    }
}

/// Creates styled spans with fuzzy match highlighting.
/// Characters at the given indices are highlighted in yellow bold.
/// Used to show which characters matched the user's search query.
fn highlight_matches(text: &str, indices: &[usize]) -> Vec<Span<'static>> {
    if indices.is_empty() {
        return vec![Span::raw(text.to_string())];
    }

    // Pre-allocate with capacity estimate (normal text + highlights)
    let mut spans = Vec::with_capacity(indices.len() * 2 + 1);
    let chars: Vec<char> = text.chars().collect();
    let mut last_idx = 0;

    // Sort and deduplicate indices to process them in order
    let mut sorted_indices = indices.to_vec();
    sorted_indices.sort_unstable();
    sorted_indices.dedup();

    let highlight_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);

    for &idx in &sorted_indices {
        if idx >= chars.len() {
            break;
        }

        // Add normal text before match (owned String)
        if idx > last_idx {
            let segment: String = chars[last_idx..idx].iter().collect();
            spans.push(Span::raw(segment));
        }

        // Add highlighted match character (owned String)
        let ch: String = chars[idx].to_string();
        spans.push(Span::styled(ch, highlight_style));

        last_idx = idx + 1;
    }

    // Add remaining text (owned String)
    if last_idx < chars.len() {
        let segment: String = chars[last_idx..].iter().collect();
        spans.push(Span::raw(segment));
    }

    if spans.is_empty() {
        spans.push(Span::raw(text.to_string()));
    }

    spans
}

fn render_split_view(f: &mut ratatui::Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(LIST_WIDTH_PERCENT),
            Constraint::Percentage(PREVIEW_WIDTH_PERCENT),
        ])
        .split(area);

    render_list(f, app, chunks[0]);
    render_preview(f, app, chunks[1]);
}

fn render_list(f: &mut ratatui::Frame, app: &mut App, area: Rect) {
    let list_width = area.width.saturating_sub(LIST_BORDER_PADDING) as usize;
    let has_search = !app.search_query.is_empty();

    let items: Vec<ListItem> = app
        .notes
        .iter()
        .enumerate()
        .map(|(idx, note)| {
            let date_str = format_date_short(&note.updated_at);
            let date_width = date_str.len();

            // Calculate available width for title
            let available_width = list_width.saturating_sub(date_width + 1); // +1 for space

            // Reduce allocations: only clone if truncation is needed
            let title_display = if note.title.len() > available_width {
                let mut truncated = String::with_capacity(available_width);
                truncated.push_str(&note.title[..available_width.saturating_sub(1)]);
                truncated.push('…');
                truncated
            } else {
                note.title.clone()
            };

            // Build line with title, spacing, and date
            let title_spans = if has_search && idx < app.match_indices.len() {
                highlight_matches(&title_display, &app.match_indices[idx])
            } else {
                vec![Span::raw(title_display.clone())]
            };

            let spacing = " ".repeat(available_width.saturating_sub(title_display.len()));

            let spans = [
                title_spans,
                vec![Span::raw(spacing), Span::styled(date_str, DARK_GRAY)],
            ]
            .concat();

            ListItem::new(vec![Line::from(spans)])
        })
        .collect();

    // Dynamic title based on search mode
    let title = if app.screen == Screen::SearchMode {
        let input = &app.input_buffer;
        format!("Search: {input}_")
    } else if app.search_query.is_empty() {
        "Notes".to_string()
    } else {
        let query = &app.search_query;
        format!("Notes (filtered: {query})")
    };

    let title_style = if app.screen == Screen::SearchMode {
        CYAN
    } else {
        Style::default()
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(border::ROUNDED)
                .title(Span::styled(title, title_style)),
        )
        .highlight_style(CYAN_BOLD)
        .highlight_symbol("▌ ");

    f.render_stateful_widget(list, area, &mut app.list_state);
}

fn render_preview(f: &mut ratatui::Frame, app: &App, area: Rect) {
    if let Some(note) = app.get_selected_note() {
        // Build metadata line
        let metadata = if note.tags.is_empty() {
            format_date_short(&note.updated_at)
        } else {
            let tags = note.tags.join(", ");
            let updated = format_date_short(&note.updated_at);
            format!("{tags} • {updated}")
        };

        // Build content with title and body using cached styles
        let content_lines = [
            vec![
                Line::from(vec![Span::styled(&note.title, CYAN_BOLD)]),
                Line::from(vec![Span::styled(metadata, DARK_GRAY)]),
                Line::from(""),
            ],
            markdown_to_lines(&note.content),
        ]
        .concat();

        // Build title with note position and scroll indicator
        let note_idx = app.list_state.selected().unwrap_or(0) + 1;
        let total_notes = app.notes.len();
        #[allow(clippy::cast_possible_truncation)]
        let content_height = content_lines.len() as u16;
        let visible_height = area.height.saturating_sub(3); // Subtract borders and padding

        let title = if app.preview_scroll > 0 {
            let max_scroll = content_height.saturating_sub(visible_height);
            let scroll_pct = if max_scroll > 0 {
                (app.preview_scroll * 100 / max_scroll).min(100)
            } else {
                0
            };
            format!("Preview [{note_idx}/{total_notes}] ↓{scroll_pct}%")
        } else {
            format!("Preview [{note_idx}/{total_notes}]")
        };

        let preview = Paragraph::new(content_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_set(border::ROUNDED)
                    .title(title),
            )
            .scroll((app.preview_scroll, 0))
            .wrap(Wrap { trim: false });
        f.render_widget(preview, area);
    } else {
        let empty = Paragraph::new("No note selected")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_set(border::ROUNDED)
                    .title("Preview"),
            )
            .style(DARK_GRAY);
        f.render_widget(empty, area);
    }
}

fn render_help(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let help_text = match app.screen {
        Screen::List => {
            let count = app.notes.len();
            let sort_info = if app.search_query.is_empty() {
                let sort_name = app.sort_mode.name();
                format!(" [{sort_name}]")
            } else {
                String::new()
            };
            format!("{count} notes{sort_info} | {HELP_LIST_MODE}")
        }
        Screen::SearchMode => {
            let found = app.notes.len();
            format!("{found} found | {HELP_SEARCH_MODE}")
        }
    };

    // Wrap to second line if needed
    let available_width = area.width as usize;
    let lines = if help_text.len() > available_width {
        // Split at last space before width
        let split_point = help_text[..available_width]
            .rfind(' ')
            .unwrap_or(available_width);

        vec![
            Line::from(help_text[..split_point].to_string()),
            Line::from(help_text[split_point..].trim().to_string()),
        ]
    } else {
        vec![Line::from(help_text)]
    };

    let help = Paragraph::new(lines).style(GRAY);
    f.render_widget(help, area);
}
