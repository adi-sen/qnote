use anyhow::Result;
use ratatui::{Terminal, crossterm::event::{self, Event, KeyEventKind}, layout::{Alignment, Constraint, Direction, Layout, Margin, Rect}, style::{Color, Modifier, Style}, symbols::border, text::{Line, Span}, widgets::{Block, Borders, List, ListItem, Paragraph, Wrap}};

use super::{app::{App, Screen}, markdown::markdown_to_lines};
use crate::{db::Note, utils::format_date_short};

const LIST_BORDER_PADDING: u16 = 4;
const UI_PADDING: u16 = 1;
const HELP_SEARCH_MODE: &str = "^n/p navigate  ⏎ accept  ESC cancel";

fn generate_help_text(app: &App) -> String {
	let kb = &app.config.keybindings;
	let batch_ops = if !app.selection.is_empty() {
		format!("⇧D batch delete ({})  ⇧X batch export ({})  ⇧C clear", app.selection.len(), app.selection.len())
	} else {
		"⇧A select all  ⇧C clear".to_string()
	};

	format!(
		"{}/{} nav  {} edit  {} new  {} del  {} search  SPC select  {} quit  ^j/k scroll  {}/{} top/bot  {} sort  {} export  ESC clear  . help  {}",
		kb.move_down,
		kb.move_up,
		kb.edit,
		kb.new_note,
		kb.delete,
		kb.search,
		kb.quit,
		kb.goto_top,
		kb.goto_bottom,
		kb.sort,
		kb.export,
		batch_ops
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
	let padded_area = f.area().inner(Margin { horizontal: UI_PADDING, vertical: 0 });
	let has_message = app.message.is_some();
	let footer_height = calculate_footer_height(app, padded_area.width);

	let constraints = if has_message {
		vec![Constraint::Min(0), Constraint::Length(1), Constraint::Length(footer_height)]
	} else {
		vec![Constraint::Min(0), Constraint::Length(footer_height)]
	};

	let chunks = Layout::default().direction(Direction::Vertical).constraints(constraints).split(padded_area);
	render_split_view(f, app, chunks[0]);

	if has_message {
		render_status_bar(f, app, chunks[1]);
		render_help(f, app, chunks[2]);
	} else {
		render_help(f, app, chunks[1]);
	}
}

fn calculate_footer_height(app: &App, width: u16) -> u16 {
	if app.screen == Screen::SearchMode {
		return 1;
	}

	if app.help_expanded {
		let help_text = generate_help_text(app);
		let available_width = width as usize;
		if help_text.len() <= available_width {
			1
		} else {
			let mut lines = 1;
			let mut remaining = help_text.len();
			while remaining > available_width {
				lines += 1;
				remaining = remaining.saturating_sub(available_width);
			}
			lines.min(3)
		}
	} else {
		1
	}
}

fn render_status_bar(f: &mut ratatui::Frame, app: &App, area: Rect) {
	if let Some(msg) = &app.message {
		f.render_widget(Paragraph::new(msg.as_str()).style(Style::default().fg(Color::Yellow)), area);
	}
}

fn highlight_title(text: &str, indices: &[usize], theme: &crate::config::ThemeConfig) -> Vec<Span<'static>> {
	if indices.is_empty() {
		return vec![Span::raw(text.to_string())];
	}

	let chars: Vec<char> = text.chars().collect();
	let mut spans = Vec::new();
	let mut sorted_indices = indices.to_vec();
	sorted_indices.sort_unstable();
	sorted_indices.dedup();

	let highlight_style = Style::default().fg(*theme.search_highlight).add_modifier(Modifier::BOLD);
	let mut last_idx = 0;

	for &idx in &sorted_indices {
		if idx >= chars.len() {
			break;
		}
		if idx > last_idx {
			spans.push(Span::raw(chars[last_idx..idx].iter().collect::<String>()));
		}
		spans.push(Span::styled(chars[idx].to_string(), highlight_style));
		last_idx = idx + 1;
	}

	if last_idx < chars.len() {
		spans.push(Span::raw(chars[last_idx..].iter().collect::<String>()));
	}

	spans
}

fn render_split_view(f: &mut ratatui::Frame, app: &mut App, area: Rect) {
	#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
	let list_percent = (app.config.ui.split_ratio * 100.0) as u16;

	let chunks = Layout::default()
		.direction(Direction::Horizontal)
		.constraints([Constraint::Percentage(list_percent), Constraint::Percentage(100 - list_percent)])
		.split(area);

	render_list(f, app, chunks[0]);
	render_preview(f, app, chunks[1]);
}

struct ListItemParams<'a> {
	note:          &'a Note,
	idx:           usize,
	is_hovered:    bool,
	is_selected:   bool,
	has_search:    bool,
	match_indices: &'a [Vec<usize>],
	list_width:    usize,
	theme:         &'a crate::config::ThemeConfig,
}

fn create_list_item(params: ListItemParams) -> ListItem<'static> {
	let ListItemParams { note, idx, is_hovered, is_selected, has_search, match_indices, list_width, theme } = params;
	let date_str = format_date_short(&note.updated_at);
	let clean_title = note.title.trim_start_matches('#').trim().to_string();

	const INDICATOR_WIDTH: usize = 2;
	let available_width = list_width.saturating_sub(date_str.len() + INDICATOR_WIDTH + 1);

	let title_display = if clean_title.len() > available_width {
		format!("{}…", &clean_title[..available_width.saturating_sub(1)])
	} else {
		clean_title.clone()
	};

	let title_spans = if has_search && idx < match_indices.len() {
		let title_len = note.title.chars().count();
		let title_indices: Vec<usize> = match_indices[idx].iter().copied().filter(|&i| i < title_len).collect();
		if title_indices.is_empty() {
			vec![Span::raw(title_display.clone())]
		} else {
			highlight_title(&title_display, &title_indices, theme)
		}
	} else {
		vec![Span::raw(title_display.clone())]
	};

	let spacing = " ".repeat(available_width.saturating_sub(title_display.len()));

	let mut spans = vec![if is_hovered && is_selected {
		Span::styled("▎ ", Style::default().fg(*theme.active_indicator).add_modifier(Modifier::BOLD))
	} else if is_hovered {
		Span::styled("▎ ", Style::default().fg(*theme.hover_indicator).add_modifier(Modifier::BOLD))
	} else if is_selected {
		Span::styled("▎ ", Style::default().fg(*theme.selection_indicator).add_modifier(Modifier::BOLD))
	} else {
		Span::raw("  ")
	}];

	let text_color = if is_selected || is_hovered { theme.text } else { theme.unselected_text };
	spans.extend(title_spans.into_iter().map(|span| {
		if span.style.fg == Some(*theme.search_highlight) {
			span
		} else {
			Span::styled(span.content, Style::default().fg(*text_color))
		}
	}));

	spans.push(Span::raw(spacing));
	spans.push(Span::styled(date_str, Style::default().fg(*theme.metadata)));

	ListItem::new(vec![Line::from(spans)])
}

fn render_list(f: &mut ratatui::Frame, app: &mut App, area: Rect) {
	let list_width = area.width.saturating_sub(LIST_BORDER_PADDING) as usize;
	let has_search = app.search.is_active();
	let current_idx = app.list_state.selected();
	let theme = &app.config.theme;

	let items: Vec<ListItem> = app
		.notes
		.iter()
		.enumerate()
		.map(|(idx, note)| {
			create_list_item(ListItemParams {
				note,
				idx,
				is_hovered: current_idx == Some(idx),
				is_selected: note.id.is_some_and(|id| app.is_note_selected(id)),
				has_search,
				match_indices: &app.search.match_indices,
				list_width,
				theme,
			})
		})
		.collect();

	let title = if app.screen == Screen::SearchMode {
		format!("Search: {}_", app.search.input_buffer)
	} else if app.search.is_active() {
		format!("Notes (search: {})", app.search.query)
	} else {
		"Notes".to_string()
	};

	let stats = if !app.selection.is_empty() {
		format!("{} notes • {} selected", app.notes.len(), app.selection.len())
	} else if app.search.is_active() {
		format!("{} matches", app.notes.len())
	} else {
		format!("{} notes • {}", app.notes.len(), app.sort_mode.name())
	};

	let title_style =
		if app.screen == Screen::SearchMode { Style::default().fg(*theme.hover_indicator) } else { Style::default() };

	let list = List::new(items)
		.block(
			Block::default()
				.borders(Borders::ALL)
				.border_set(border::ROUNDED)
				.title(Span::styled(title, title_style))
				.title_bottom(Span::styled(stats, Style::default().fg(*theme.metadata))),
		)
		.highlight_style(Style::default())
		.highlight_symbol("");

	f.render_stateful_widget(list, area, &mut app.list_state);
}

fn render_preview(f: &mut ratatui::Frame, app: &App, area: Rect) {
	let theme = &app.config.theme;
	let overlay_color = Style::default().fg(*theme.metadata);

	if let Some(note) = app.get_selected_note() {
		let metadata = if note.tags.is_empty() {
			format_date_short(&note.updated_at)
		} else {
			format!("{} • {}", note.tags.join(", "), format_date_short(&note.updated_at))
		};

		let clean_title = note.title.trim_start_matches('#').trim();
		let teal_bold = Style::default().fg(*theme.hover_indicator).add_modifier(Modifier::BOLD);

		let content_lines = vec![
			Line::from(vec![Span::raw("  "), Span::styled(clean_title, teal_bold)]),
			Line::from(vec![Span::raw("  "), Span::styled(metadata, overlay_color)]),
			Line::from(""),
		]
		.into_iter()
		.chain(markdown_to_lines(&note.content, theme).into_iter().map(|line| {
			let mut padded_spans = vec![Span::raw("  ")];
			padded_spans.extend(line.spans);
			Line::from(padded_spans)
		}))
		.collect::<Vec<_>>();

		let note_idx = app.list_state.selected().unwrap_or(0) + 1;
		#[allow(clippy::cast_possible_truncation)]
		let content_height = content_lines.len() as u16;
		let visible_height = area.height.saturating_sub(3);

		let scroll_indicator = if app.preview_scroll > 0 {
			let max_scroll = content_height.saturating_sub(visible_height);
			let scroll_pct = if max_scroll > 0 { (app.preview_scroll * 100 / max_scroll).min(100) } else { 0 };
			format!(" ↓{}%", scroll_pct)
		} else {
			String::new()
		};

		let block = Block::default()
			.borders(Borders::ALL)
			.border_set(border::ROUNDED)
			.title(format!("Preview{}", scroll_indicator))
			.title_bottom(Span::styled(format!("{}/{}", note_idx, app.notes.len()), overlay_color));

		f.render_widget(
			Paragraph::new(content_lines).block(block).scroll((app.preview_scroll, 0)).wrap(Wrap { trim: false }),
			area,
		);
	} else {
		f.render_widget(
			Paragraph::new("No note selected")
				.block(Block::default().borders(Borders::ALL).border_set(border::ROUNDED).title("Preview"))
				.style(overlay_color),
			area,
		);
	}
}

fn render_help(f: &mut ratatui::Frame, app: &App, area: Rect) {
	let theme = &app.config.theme;
	let help_color = Style::default().fg(*theme.metadata);

	let lines = match app.screen {
		Screen::List => {
			let help_text = generate_help_text(app);
			let available_width = area.width as usize;

			if !app.help_expanded {
				if help_text.len() <= available_width {
					vec![Line::from(Span::styled(help_text, help_color))]
				} else {
					let truncate_at = available_width.saturating_sub(7);
					let break_point = help_text[..truncate_at.min(help_text.len())].rfind(' ').unwrap_or(truncate_at);
					vec![Line::from(Span::styled(format!("{} . more", &help_text[..break_point]), help_color))]
				}
			} else {
				let mut lines = Vec::new();
				let mut remaining = help_text.as_str();
				while !remaining.is_empty() {
					if remaining.len() <= available_width {
						lines.push(Line::from(Span::styled(remaining.to_string(), help_color)));
						break;
					}
					let split_at = remaining[..available_width.min(remaining.len())].rfind(' ').unwrap_or(available_width);
					lines.push(Line::from(Span::styled(remaining[..split_at].to_string(), help_color)));
					remaining = remaining[split_at..].trim_start();
				}
				lines
			}
		}
		Screen::SearchMode => vec![Line::from(Span::styled(HELP_SEARCH_MODE, help_color))],
	};

	f.render_widget(Paragraph::new(lines).alignment(Alignment::Center), area);
}
