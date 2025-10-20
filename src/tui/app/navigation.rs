use crate::{config::UiConfig, db::Note};

/// Estimates the height of the preview content for scroll bounds checking.
pub fn get_preview_content_height(note: &Note, ui_config: &UiConfig) -> u16 {
	#[allow(clippy::cast_possible_truncation)]
	let lines = note.content.lines().count() as u16;
	#[allow(clippy::cast_possible_truncation)]
	let headers = (note.content.matches('#').count() as u16).min(ui_config.max_markdown_formatting_buffer);

	ui_config.header_lines + lines + headers
}

/// Scroll the preview pane up or down.
pub fn scroll_preview(scroll: &mut u16, down: bool, content_height: u16, ui_config: &UiConfig) {
	if down {
		let max_scroll = content_height.saturating_sub(ui_config.preview_max_scroll_buffer);
		*scroll = (*scroll + ui_config.preview_scroll_step).min(max_scroll);
	} else {
		*scroll = scroll.saturating_sub(ui_config.preview_scroll_step);
	}
}
