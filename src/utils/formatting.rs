//! String and date formatting utilities.

use chrono::{DateTime, Utc};

/// Date format constants for consistent formatting across the application.
pub mod date_formats {
	/// Short format for list views: "Jan 15"
	pub const SHORT: &str = "%b %d";

	/// Full format for detailed views: "2024-01-15 14:30"
	pub const FULL: &str = "%Y-%m-%d %H:%M";

	/// Date only format: "2024-01-15"
	pub const DATE_ONLY: &str = "%Y-%m-%d";
}

/// Sanitizes a note title for use as a filename.
/// Replaces '/' with '-' and spaces with '_' to create filesystem-safe names.
///
/// # Examples
/// ```
/// assert_eq!(sanitize_filename("My Note"), "My_Note");
/// assert_eq!(sanitize_filename("Path/To/Note"), "Path-To-Note");
/// ```
pub fn sanitize_filename(title: &str) -> String { title.replace('/', "-").replace(' ', "_") }

/// Formats a datetime for list view display (short format).
/// Returns: "Jan 15"
pub fn format_date_short(dt: &DateTime<Utc>) -> String { dt.format(date_formats::SHORT).to_string() }

/// Formats a datetime for detailed view display (full format).
/// Returns: "2024-01-15 14:30"
pub fn format_date_full(dt: &DateTime<Utc>) -> String { dt.format(date_formats::FULL).to_string() }

/// Formats a datetime as date only (no time).
/// Returns: "2024-01-15"
pub fn format_date_only(dt: &DateTime<Utc>) -> String { dt.format(date_formats::DATE_ONLY).to_string() }
