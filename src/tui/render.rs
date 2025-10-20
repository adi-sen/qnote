use anyhow::Result;
use ratatui::{Terminal, crossterm::event::{self, Event, KeyEventKind}, layout::{Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style}, symbols::border, text::{Line, Span}, widgets::{Block, Borders, List, ListItem, Paragraph, Wrap}};

use super::{app::{App, Screen}, markdown::markdown_to_lines};
use crate::utils::format_date_short;

// UI layout constants
const LIST_BORDER_PADDING: u16 = 4;

const CYAN_BOLD: Style = Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD);
const CYAN: Style = Style::new().fg(Color::Cyan);
const DARK_GRAY: Style = Style::new().fg(Color::DarkGray);
const GRAY: Style = Style::new().fg(Color::Gray);

const HELP_SEARCH_MODE: &str = "^n/p navigate  ⏎ accept  ESC cancel";

/// Generate dynamic help text based on current keybindings
fn generate_help_text(app: &App) -> String {
	let kb = &app.config.keybindings;
	format!(
		"{}/{} navigate  ^j/k scroll  {}/{} top/bottom  ⏎ edit  {} new  {} delete  {} sort  {} export  {} search  ESC clear  {} quit",
		kb.move_down,
		kb.move_up,
		kb.goto_top,
		kb.goto_bottom,
		kb.new_note,
		kb.delete,
		kb.sort,
		kb.export,
		kb.search,
		kb.quit
	)
}

pub fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
	loop {
		if app.needs_clear {
			terminal.clear()?;
			app.needs_clear = false;
		}
		terminal.draw(|f| ui(f, app))?;

		if let Event::Key(key) = event::read()?
			&& key.kind == KeyEventKind::Press
		{
			app.tick_message();

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

	let chunks = Layout::default().direction(Direction::Vertical).constraints(constraints).split(f.area());
	render_split_view(f, app, chunks[0]);

	if has_message {
		render_status_bar(f, app, chunks[1]);
		render_help(f, app, chunks[2]);
	} else {
		render_help(f, app, chunks[1]);
	}
}

fn calculate_footer_height(app: &App, width: u16) -> u16 {
	let help_text = match app.screen {
		Screen::List => generate_help_text(app),
		Screen::SearchMode => HELP_SEARCH_MODE.to_string(),
	};
	if help_text.len() > width as usize { 2 } else { 1 }
}

fn render_status_bar(f: &mut ratatui::Frame, app: &App, area: Rect) {
	if let Some(msg) = &app.message {
		let status = Paragraph::new(msg.as_str()).style(Style::default().fg(Color::Yellow));
		f.render_widget(status, area);
	}
}

/// Simple highlighting for matched characters in title.
fn highlight_title(text: &str, indices: &[usize]) -> Vec<Span<'static>> {
	if indices.is_empty() {
		return vec![Span::raw(text.to_string())];
	}

	let chars: Vec<char> = text.chars().collect();
	let mut spans = Vec::with_capacity(indices.len() * 2 + 1);
	let mut sorted_indices = indices.to_vec();
	sorted_indices.sort_unstable();
	sorted_indices.dedup();

	let highlight_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
	let mut last_idx = 0;

	for &idx in &sorted_indices {
		if idx >= chars.len() {
			break;
		}

		if idx > last_idx {
			let segment: String = chars[last_idx..idx].iter().collect();
			spans.push(Span::raw(segment));
		}

		let ch: String = chars[idx].to_string();
		spans.push(Span::styled(ch, highlight_style));
		last_idx = idx + 1;
	}

	if last_idx < chars.len() {
		let segment: String = chars[last_idx..].iter().collect();
		spans.push(Span::raw(segment));
	}

	spans
}

fn render_split_view(f: &mut ratatui::Frame, app: &mut App, area: Rect) {
	// Calculate split percentages from config (split_ratio is for list pane)
	#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
	let list_percent = (app.config.ui.split_ratio * 100.0) as u16;
	let preview_percent = 100 - list_percent;

	let chunks = Layout::default()
		.direction(Direction::Horizontal)
		.constraints([Constraint::Percentage(list_percent), Constraint::Percentage(preview_percent)])
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
			let available_width = list_width.saturating_sub(date_width + 1);

			// Truncate title if needed
			let title_display = if note.title.len() > available_width {
				let mut truncated = String::with_capacity(available_width);
				truncated.push_str(&note.title[..available_width.saturating_sub(1)]);
				truncated.push('…');
				truncated
			} else {
				note.title.clone()
			};

			// Apply simple highlighting in title if searching
			let title_spans = if has_search && idx < app.match_indices.len() {
				let title_len = note.title.chars().count();
				let title_indices: Vec<usize> =
					app.match_indices[idx].iter().filter_map(|&i| if i < title_len { Some(i) } else { None }).collect();

				if !title_indices.is_empty() {
					highlight_title(&title_display, &title_indices)
				} else {
					vec![Span::raw(title_display.clone())]
				}
			} else {
				vec![Span::raw(title_display.clone())]
			};

			let spacing = " ".repeat(available_width.saturating_sub(title_display.len()));
			let mut spans = title_spans;
			spans.push(Span::raw(spacing));
			spans.push(Span::styled(date_str, DARK_GRAY));

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
		format!("Notes (search: {query})")
	};

	let title_style = if app.screen == Screen::SearchMode { CYAN } else { Style::default() };

	let list = List::new(items)
		.block(Block::default().borders(Borders::ALL).border_set(border::ROUNDED).title(Span::styled(title, title_style)))
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

		// Build content with title and body
		let content_lines = vec![
			Line::from(vec![Span::styled(&note.title, CYAN_BOLD)]),
			Line::from(vec![Span::styled(metadata, DARK_GRAY)]),
			Line::from(""),
		]
		.into_iter()
		.chain(markdown_to_lines(&note.content))
		.collect::<Vec<_>>();

		// Build title with note position and scroll indicator
		let note_idx = app.list_state.selected().unwrap_or(0) + 1;
		let total_notes = app.notes.len();
		#[allow(clippy::cast_possible_truncation)]
		let content_height = content_lines.len() as u16;
		let visible_height = area.height.saturating_sub(3); // Subtract borders and padding

		let title = if app.preview_scroll > 0 {
			let max_scroll = content_height.saturating_sub(visible_height);
			let scroll_pct = if max_scroll > 0 { (app.preview_scroll * 100 / max_scroll).min(100) } else { 0 };
			format!("Preview [{note_idx}/{total_notes}] ↓{scroll_pct}%")
		} else {
			format!("Preview [{note_idx}/{total_notes}]")
		};

		let preview = Paragraph::new(content_lines)
			.block(Block::default().borders(Borders::ALL).border_set(border::ROUNDED).title(title))
			.scroll((app.preview_scroll, 0))
			.wrap(Wrap { trim: false });
		f.render_widget(preview, area);
	} else {
		let empty = Paragraph::new("No note selected")
			.block(Block::default().borders(Borders::ALL).border_set(border::ROUNDED).title("Preview"))
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
			format!("{count} notes{sort_info} | {}", generate_help_text(app))
		}
		Screen::SearchMode => {
			let found = app.notes.len();
			format!("{found} matches | {HELP_SEARCH_MODE}")
		}
	};

	// Wrap to second line if needed
	let available_width = area.width as usize;
	let lines = if help_text.len() > available_width {
		// Split at last space before width
		let split_point = help_text[..available_width].rfind(' ').unwrap_or(available_width);

		vec![Line::from(help_text[..split_point].to_string()), Line::from(help_text[split_point..].trim().to_string())]
	} else {
		vec![Line::from(help_text)]
	};

	let help = Paragraph::new(lines).style(GRAY);
	f.render_widget(help, area);
}
