use std::ops::Deref;

use ratatui::style::Color;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Wrapper for Color with custom serde implementation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeColor(Color);

impl ThemeColor {
	const fn new(color: Color) -> Self { Self(color) }
}

impl Deref for ThemeColor {
	type Target = Color;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<ThemeColor> for Color {
	fn from(tc: ThemeColor) -> Self { tc.0 }
}

impl Serialize for ThemeColor {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let hex = color_to_hex(&self.0);
		serializer.serialize_str(&hex)
	}
}

impl<'de> Deserialize<'de> for ThemeColor {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;
		let color = parse_color(&s).map_err(serde::de::Error::custom)?;
		Ok(Self(color))
	}
}

/// Theme configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ThemeConfig {
	// UI elements
	#[serde(default = "default_text")]
	pub text:                ThemeColor,
	#[serde(default = "default_unselected_text")]
	pub unselected_text:     ThemeColor,
	#[serde(default = "default_metadata")]
	pub metadata:            ThemeColor,
	#[serde(default = "default_hover_indicator")]
	pub hover_indicator:     ThemeColor,
	#[serde(default = "default_selection_indicator")]
	pub selection_indicator: ThemeColor,
	#[serde(default = "default_active_indicator")]
	pub active_indicator:    ThemeColor,
	#[serde(default = "default_search_highlight")]
	pub search_highlight:    ThemeColor,

	// Markdown - Headings
	#[serde(default = "default_h1")]
	pub h1:    ThemeColor,
	#[serde(default = "default_h2")]
	pub h2:    ThemeColor,
	#[serde(default = "default_h3")]
	pub h3:    ThemeColor,
	#[serde(default = "default_h4_h6")]
	pub h4_h6: ThemeColor,

	// Markdown - Code
	#[serde(default = "default_code")]
	pub code:       ThemeColor,
	#[serde(default = "default_code_block")]
	pub code_block: ThemeColor,

	// Markdown - Text styles
	#[serde(default = "default_link")]
	pub link:          ThemeColor,
	#[serde(default = "default_emphasis")]
	pub emphasis:      ThemeColor,
	#[serde(default = "default_strong")]
	pub strong:        ThemeColor,
	#[serde(default = "default_strikethrough")]
	pub strikethrough: ThemeColor,
	#[serde(default = "default_blockquote")]
	pub blockquote:    ThemeColor,
}

// Tokyo Night defaults
const fn default_text() -> ThemeColor { ThemeColor::new(Color::Rgb(0xc0, 0xca, 0xf5)) }
const fn default_unselected_text() -> ThemeColor { ThemeColor::new(Color::Rgb(0x56, 0x5f, 0x89)) }
const fn default_metadata() -> ThemeColor { ThemeColor::new(Color::Rgb(0x56, 0x5f, 0x89)) }
const fn default_hover_indicator() -> ThemeColor { ThemeColor::new(Color::Rgb(0x7a, 0xa2, 0xf7)) }
const fn default_selection_indicator() -> ThemeColor { ThemeColor::new(Color::Rgb(0xe0, 0xaf, 0x68)) }
const fn default_active_indicator() -> ThemeColor { ThemeColor::new(Color::Rgb(0xff, 0x9e, 0x64)) }
const fn default_search_highlight() -> ThemeColor { ThemeColor::new(Color::Rgb(0xbb, 0x9a, 0xf7)) }
const fn default_h1() -> ThemeColor { ThemeColor::new(Color::Rgb(0x7d, 0xcf, 0xff)) }
const fn default_h2() -> ThemeColor { ThemeColor::new(Color::Rgb(0x7a, 0xa2, 0xf7)) }
const fn default_h3() -> ThemeColor { ThemeColor::new(Color::Rgb(0x7d, 0xcf, 0xff)) }
const fn default_h4_h6() -> ThemeColor { ThemeColor::new(Color::Rgb(0x7a, 0xa2, 0xf7)) }
const fn default_code() -> ThemeColor { ThemeColor::new(Color::Rgb(0x9e, 0xce, 0x6a)) }
const fn default_code_block() -> ThemeColor { ThemeColor::new(Color::Rgb(0x9e, 0xce, 0x6a)) }
const fn default_link() -> ThemeColor { ThemeColor::new(Color::Rgb(0x7a, 0xa2, 0xf7)) }
const fn default_emphasis() -> ThemeColor { ThemeColor::new(Color::Rgb(0xff, 0x9e, 0x64)) }
const fn default_strong() -> ThemeColor { ThemeColor::new(Color::Rgb(0xc0, 0xca, 0xf5)) }
const fn default_strikethrough() -> ThemeColor { ThemeColor::new(Color::Rgb(0x56, 0x5f, 0x89)) }
const fn default_blockquote() -> ThemeColor { ThemeColor::new(Color::Rgb(0x56, 0x5f, 0x89)) }

impl Default for ThemeConfig {
	fn default() -> Self { Self::tokyo_night() }
}

impl ThemeConfig {
	const fn tokyo_night() -> Self {
		Self {
			text:                default_text(),
			unselected_text:     default_unselected_text(),
			metadata:            default_metadata(),
			hover_indicator:     default_hover_indicator(),
			selection_indicator: default_selection_indicator(),
			active_indicator:    default_active_indicator(),
			search_highlight:    default_search_highlight(),
			h1:                  default_h1(),
			h2:                  default_h2(),
			h3:                  default_h3(),
			h4_h6:               default_h4_h6(),
			code:                default_code(),
			code_block:          default_code_block(),
			link:                default_link(),
			emphasis:            default_emphasis(),
			strong:              default_strong(),
			strikethrough:       default_strikethrough(),
			blockquote:          default_blockquote(),
		}
	}
}

/// Parse color from various string formats
fn parse_color(s: &str) -> Result<Color, String> {
	let s = s.trim().to_lowercase();

	// Hex color: #RRGGBB or #RGB
	if let Some(stripped) = s.strip_prefix('#') {
		return parse_hex_color(stripped);
	}

	// RGB: rgb(r, g, b)
	if s.starts_with("rgb(") && s.ends_with(')') {
		return parse_rgb_color(&s[4..s.len() - 1]);
	}

	// Indexed: "10" or "255"
	if let Ok(idx) = s.parse::<u8>() {
		return Ok(Color::Indexed(idx));
	}

	// Named colors
	match s.as_str() {
		"black" => Ok(Color::Black),
		"red" => Ok(Color::Red),
		"green" => Ok(Color::Green),
		"yellow" => Ok(Color::Yellow),
		"blue" => Ok(Color::Blue),
		"magenta" => Ok(Color::Magenta),
		"cyan" => Ok(Color::Cyan),
		"gray" | "grey" => Ok(Color::Gray),
		"darkgray" | "darkgrey" => Ok(Color::DarkGray),
		"lightred" => Ok(Color::LightRed),
		"lightgreen" => Ok(Color::LightGreen),
		"lightyellow" => Ok(Color::LightYellow),
		"lightblue" => Ok(Color::LightBlue),
		"lightmagenta" => Ok(Color::LightMagenta),
		"lightcyan" => Ok(Color::LightCyan),
		"white" => Ok(Color::White),
		"reset" => Ok(Color::Reset),
		_ => Err(format!("Unknown color: '{}'", s)),
	}
}

fn parse_hex_color(hex: &str) -> Result<Color, String> {
	let hex = hex.trim();

	// Handle #RGB format (shorthand)
	if hex.len() == 3 {
		let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).map_err(|e| format!("Invalid hex: {}", e))?;
		let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).map_err(|e| format!("Invalid hex: {}", e))?;
		let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).map_err(|e| format!("Invalid hex: {}", e))?;
		return Ok(Color::Rgb(r, g, b));
	}

	// Handle #RRGGBB format
	if hex.len() == 6 {
		let r = u8::from_str_radix(&hex[0..2], 16).map_err(|e| format!("Invalid hex: {}", e))?;
		let g = u8::from_str_radix(&hex[2..4], 16).map_err(|e| format!("Invalid hex: {}", e))?;
		let b = u8::from_str_radix(&hex[4..6], 16).map_err(|e| format!("Invalid hex: {}", e))?;
		return Ok(Color::Rgb(r, g, b));
	}

	Err(format!("Invalid hex color length: {} (expected 3 or 6)", hex.len()))
}

fn parse_rgb_color(rgb: &str) -> Result<Color, String> {
	let parts: Vec<&str> = rgb.split(',').map(|s| s.trim()).collect();

	if parts.len() != 3 {
		return Err("RGB format must be: rgb(r, g, b)".to_string());
	}

	let r = parts[0].parse::<u8>().map_err(|e| format!("Invalid red value: {}", e))?;
	let g = parts[1].parse::<u8>().map_err(|e| format!("Invalid green value: {}", e))?;
	let b = parts[2].parse::<u8>().map_err(|e| format!("Invalid blue value: {}", e))?;

	Ok(Color::Rgb(r, g, b))
}

/// Convert Color to hex string for serialization
pub(super) fn color_to_hex(color: &Color) -> String {
	match color {
		Color::Rgb(r, g, b) => format!("#{:02x}{:02x}{:02x}", r, g, b),
		Color::Black => "black".to_string(),
		Color::Red => "red".to_string(),
		Color::Green => "green".to_string(),
		Color::Yellow => "yellow".to_string(),
		Color::Blue => "blue".to_string(),
		Color::Magenta => "magenta".to_string(),
		Color::Cyan => "cyan".to_string(),
		Color::Gray => "gray".to_string(),
		Color::DarkGray => "darkgray".to_string(),
		Color::LightRed => "lightred".to_string(),
		Color::LightGreen => "lightgreen".to_string(),
		Color::LightYellow => "lightyellow".to_string(),
		Color::LightBlue => "lightblue".to_string(),
		Color::LightMagenta => "lightmagenta".to_string(),
		Color::LightCyan => "lightcyan".to_string(),
		Color::White => "white".to_string(),
		Color::Indexed(i) => i.to_string(),
		Color::Reset => "reset".to_string(),
	}
}
