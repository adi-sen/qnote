mod database;
mod defaults;
mod editor;
mod keybindings;
mod ui;

use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
pub use database::DatabaseConfig;
pub use editor::EditorConfig;
pub use keybindings::KeybindingsConfig;
use serde::{Deserialize, Serialize};
pub use ui::UiConfig;

/// Configuration for the qnote application.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
	#[serde(default)]
	pub ui:          UiConfig,
	#[serde(default)]
	pub editor:      EditorConfig,
	#[serde(default)]
	pub keybindings: KeybindingsConfig,
	#[serde(default)]
	pub database:    DatabaseConfig,
}

impl Config {
	/// Loads configuration from the default config file path.
	/// Returns default config if file doesn't exist (following convention of bat,
	/// starship, etc.)
	pub fn load() -> Result<Self> {
		let config_path = Self::get_config_path()?;

		if !config_path.exists() {
			// Use defaults without creating a file (standard Rust CLI tool behavior)
			return Ok(Self::default());
		}

		let config_str = fs::read_to_string(&config_path).context("Failed to read config file")?;

		toml::from_str(&config_str).context("Failed to parse config file")
	}

	/// Saves the configuration to the default config file path with inline
	/// comments.
	pub fn save(&self) -> Result<()> {
		let config_path = Self::get_config_path()?;

		// Ensure parent directory exists
		if let Some(parent) = config_path.parent() {
			fs::create_dir_all(parent).context("Failed to create config directory")?;
		}

		// Generate TOML with inline comments
		let config_with_comments = self.to_toml_with_comments();

		fs::write(&config_path, config_with_comments).context("Failed to write config file")?;

		Ok(())
	}

	/// Generates TOML string with helpful inline comments for each field
	fn to_toml_with_comments(&self) -> String {
		format!(
			r#"# qnote configuration file
# This file is automatically created on first run
# Edit this file to customize qnote's behavior

[ui]
# List pane width (0.1-0.9). Example: 0.3 = 30% list, 70% preview
split_ratio = {split_ratio}
# Number of keypresses before status messages disappear
message_display_keypresses = {message_display_keypresses}
# Lines to scroll in preview with Ctrl+j/k
preview_scroll_step = {preview_scroll_step}
# Preview scroll buffer for maximum scroll bounds
preview_max_scroll_buffer = {preview_max_scroll_buffer}
# Number of header lines in preview (title + metadata + blank)
header_lines = {header_lines}
# Maximum markdown formatting buffer for height calculation
max_markdown_formatting_buffer = {max_markdown_formatting_buffer}

[editor]
# Override $EDITOR environment variable (optional, remove to use $EDITOR)
{default_editor}# Secure temp files with 0600 permissions (Unix only)
secure_temp_files = {secure_temp_files}

[keybindings]
# Customize keyboard shortcuts (single characters only)
quit = "{quit}"
new_note = "{new_note}"
delete = "{delete}"
edit = "{edit}"
search = "{search}"
export = "{export}"
sort = "{sort}"
goto_top = "{goto_top}"
goto_bottom = "{goto_bottom}"
move_down = "{move_down}"
move_up = "{move_up}"

[database]
# Enable Write-Ahead Logging for better performance (disable for network drives)
wal_mode = {wal_mode}
# Database cache size in KB (negative = KB, positive = pages). Default: -64000 (64MB)
cache_size_kb = {cache_size_kb}
# Synchronous mode: OFF, NORMAL (default), FULL, or EXTRA
synchronous = "{synchronous}"
# Temp store: DEFAULT, FILE, or MEMORY (default)
temp_store = "{temp_store}"
"#,
			split_ratio = self.ui.split_ratio,
			message_display_keypresses = self.ui.message_display_keypresses,
			preview_scroll_step = self.ui.preview_scroll_step,
			preview_max_scroll_buffer = self.ui.preview_max_scroll_buffer,
			header_lines = self.ui.header_lines,
			max_markdown_formatting_buffer = self.ui.max_markdown_formatting_buffer,
			default_editor = if let Some(ref editor) = self.editor.default_editor {
				format!("default_editor = \"{}\"\n", editor)
			} else {
				"# default_editor = \"nvim\"\n".to_string()
			},
			secure_temp_files = self.editor.secure_temp_files,
			quit = self.keybindings.quit,
			new_note = self.keybindings.new_note,
			delete = self.keybindings.delete,
			edit = self.keybindings.edit,
			search = self.keybindings.search,
			export = self.keybindings.export,
			sort = self.keybindings.sort,
			goto_top = self.keybindings.goto_top,
			goto_bottom = self.keybindings.goto_bottom,
			move_down = self.keybindings.move_down,
			move_up = self.keybindings.move_up,
			wal_mode = self.database.wal_mode,
			cache_size_kb = self.database.cache_size_kb,
			synchronous = self.database.synchronous,
			temp_store = self.database.temp_store,
		)
	}

	/// Returns the platform-specific configuration file path.
	/// On Unix: ~/.config/qnote/config.toml
	/// On Windows: %APPDATA%\qnote\config.toml
	pub fn get_config_path() -> Result<PathBuf> {
		let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
		path.push("qnote");
		path.push("config.toml");
		Ok(path)
	}

	/// Validates the configuration values.
	pub fn validate(&self) -> Result<()> {
		if !(0.1..=0.9).contains(&self.ui.split_ratio) {
			anyhow::bail!("ui.split_ratio must be between 0.1 and 0.9");
		}

		if self.ui.message_display_keypresses == 0 {
			anyhow::bail!("ui.message_display_keypresses must be greater than 0");
		}

		if self.ui.preview_scroll_step == 0 {
			anyhow::bail!("ui.preview_scroll_step must be greater than 0");
		}

		if self.ui.preview_max_scroll_buffer == 0 {
			anyhow::bail!("ui.preview_max_scroll_buffer must be greater than 0");
		}

		if self.ui.header_lines == 0 {
			anyhow::bail!("ui.header_lines must be greater than 0");
		}

		// Validate database synchronous mode
		let valid_sync_modes = ["OFF", "NORMAL", "FULL", "EXTRA"];
		if !valid_sync_modes.contains(&self.database.synchronous.as_str()) {
			anyhow::bail!("database.synchronous must be one of: {}", valid_sync_modes.join(", "));
		}

		// Validate database temp store
		let valid_temp_stores = ["DEFAULT", "FILE", "MEMORY"];
		if !valid_temp_stores.contains(&self.database.temp_store.as_str()) {
			anyhow::bail!("database.temp_store must be one of: {}", valid_temp_stores.join(", "));
		}

		Ok(())
	}
}
