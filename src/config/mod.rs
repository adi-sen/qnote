mod database;
mod defaults;
mod editor;
mod keybindings;
mod theme;
mod ui;

use std::{env, fs, path::PathBuf};

use anyhow::{Context, Result};
pub use database::DatabaseConfig;
pub use editor::EditorConfig;
pub use keybindings::KeybindingsConfig;
use serde::{Deserialize, Serialize};
pub use theme::ThemeConfig;
use theme::color_to_hex;
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
	#[serde(default)]
	pub theme:       ThemeConfig,
}

impl Config {
	/// Loads configuration from the default config file path.
	/// Auto-generates config with defaults on first run.
	pub fn load() -> Result<Self> {
		let config_path = Self::get_config_path()?;

		if !config_path.exists() {
			let config = Self::default();
			config.save()?;
			return Ok(config);
		}

		let config_str = fs::read_to_string(&config_path).context("Failed to read config file")?;

		let config: Self = toml::from_str(&config_str).context("Failed to parse config file")?;
		config.validate()?;
		Ok(config)
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
# Edit this file to customize qnote's behavior

[theme]
# UI colors
text = "{text}"
unselected_text = "{unselected_text}"
metadata = "{metadata}"
hover_indicator = "{hover_indicator}"
selection_indicator = "{selection_indicator}"
active_indicator = "{active_indicator}"
search_highlight = "{search_highlight}"

# Markdown headings
h1 = "{h1}"
h2 = "{h2}"
h3 = "{h3}"
h4_h6 = "{h4_h6}"

# Markdown code
code = "{code}"
code_block = "{code_block}"

# Markdown text styles
link = "{link}"
emphasis = "{emphasis}"
strong = "{strong}"
strikethrough = "{strikethrough}"
blockquote = "{blockquote}"

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
{default_editor}{secure_temp_files}

[database]
# Enable Write-Ahead Logging for better performance (disable for network drives)
wal_mode = {wal_mode}
# Database cache size in kilobytes (negative value = KB, positive = pages)
cache_size_kb = {cache_size_kb}
# Synchronous mode: OFF, NORMAL, FULL, or EXTRA
synchronous = "{synchronous}"
# Temp store: DEFAULT, FILE, or MEMORY
temp_store = "{temp_store}"

[keybindings]
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
"#,
			text = color_to_hex(&self.theme.text),
			unselected_text = color_to_hex(&self.theme.unselected_text),
			metadata = color_to_hex(&self.theme.metadata),
			hover_indicator = color_to_hex(&self.theme.hover_indicator),
			selection_indicator = color_to_hex(&self.theme.selection_indicator),
			active_indicator = color_to_hex(&self.theme.active_indicator),
			search_highlight = color_to_hex(&self.theme.search_highlight),
			h1 = color_to_hex(&self.theme.h1),
			h2 = color_to_hex(&self.theme.h2),
			h3 = color_to_hex(&self.theme.h3),
			h4_h6 = color_to_hex(&self.theme.h4_h6),
			code = color_to_hex(&self.theme.code),
			code_block = color_to_hex(&self.theme.code_block),
			link = color_to_hex(&self.theme.link),
			emphasis = color_to_hex(&self.theme.emphasis),
			strong = color_to_hex(&self.theme.strong),
			strikethrough = color_to_hex(&self.theme.strikethrough),
			blockquote = color_to_hex(&self.theme.blockquote),
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
			secure_temp_files = if self.editor.secure_temp_files {
				"# secure_temp_files = true\n".to_string()
			} else {
				"secure_temp_files = false\n".to_string()
			},
			wal_mode = self.database.wal_mode,
			cache_size_kb = self.database.cache_size_kb,
			synchronous = self.database.synchronous,
			temp_store = self.database.temp_store,
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
		)
	}

	/// Returns the platform-specific configuration file path following XDG spec.
	/// Priority order:
	/// 1. $XDG_CONFIG_HOME/qnote/config.toml
	/// 2. ~/.config/qnote/config.toml (Unix)
	/// 3. ~/Library/Application Support/qnote/config.toml (macOS fallback)
	/// 4. %APPDATA%\qnote\config.toml (Windows)
	pub fn get_config_path() -> Result<PathBuf> {
		let config_dir = if let Ok(xdg_config) = env::var("XDG_CONFIG_HOME") {
			// Use XDG_CONFIG_HOME if set
			PathBuf::from(xdg_config)
		} else if cfg!(target_os = "macos") {
			// On macOS, prefer ~/.config but fall back to Application Support
			let home = dirs::home_dir().context("Failed to get home directory")?;
			let xdg_path = home.join(".config");
			if xdg_path.exists() {
				xdg_path
			} else {
				// Fall back to Application Support on macOS
				dirs::config_dir().unwrap_or_else(|| home.join("Library/Application Support"))
			}
		} else {
			// On other platforms, use standard config dir
			dirs::config_dir().context("Failed to get config directory")?
		};

		Ok(config_dir.join("qnote").join("config.toml"))
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
