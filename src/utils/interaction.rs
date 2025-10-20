//! User interaction utilities.

use std::io::{Write, stdin, stdout};

/// Prompts user for confirmation. Returns true if user confirms.
///
/// # Examples
/// ```
/// if confirm("Delete this note?") {
///     // User confirmed
/// }
/// ```
pub fn confirm(prompt: &str) -> bool {
	print!("{prompt} (y/N): ");
	stdout().flush().ok();
	let mut input = String::new();
	stdin().read_line(&mut input).ok();
	matches!(input.trim(), "y" | "Y" | "yes" | "Yes")
}
