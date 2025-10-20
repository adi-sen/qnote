use serde::{Deserialize, Serialize};

/// UI-related configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
	/// List pane width as percentage (0.1-0.9). Example: 0.3 = 30% list, 70%
	/// preview
	#[serde(default = "default_split_ratio")]
	pub split_ratio: f32,

	/// Number of keypresses before auto-clearing status messages
	#[serde(default = "default_message_display_keypresses")]
	pub message_display_keypresses: u8,

	/// Number of lines to scroll in preview pane with Ctrl+j/k
	#[serde(default = "default_preview_scroll_step")]
	pub preview_scroll_step: u16,

	/// Preview scroll buffer for maximum scroll bounds
	#[serde(default = "default_preview_max_scroll_buffer")]
	pub preview_max_scroll_buffer: u16,

	/// Number of header lines in preview (title + metadata + blank)
	#[serde(default = "default_header_lines")]
	pub header_lines: u16,

	/// Maximum markdown formatting buffer for height calculation
	#[serde(default = "default_max_markdown_formatting_buffer")]
	pub max_markdown_formatting_buffer: u16,
}

const fn default_split_ratio() -> f32 { 0.4 }

const fn default_message_display_keypresses() -> u8 { 5 }

const fn default_preview_scroll_step() -> u16 { 3 }

const fn default_preview_max_scroll_buffer() -> u16 { 10 }

const fn default_header_lines() -> u16 { 3 }

const fn default_max_markdown_formatting_buffer() -> u16 { 10 }

impl Default for UiConfig {
	fn default() -> Self {
		Self {
			split_ratio:                    default_split_ratio(),
			message_display_keypresses:     default_message_display_keypresses(),
			preview_scroll_step:            default_preview_scroll_step(),
			preview_max_scroll_buffer:      default_preview_max_scroll_buffer(),
			header_lines:                   default_header_lines(),
			max_markdown_formatting_buffer: default_max_markdown_formatting_buffer(),
		}
	}
}
