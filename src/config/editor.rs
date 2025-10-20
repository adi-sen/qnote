use serde::{Deserialize, Serialize};

use super::defaults::default_true;

/// Editor-related configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
	/// Default editor command (overrides $EDITOR)
	#[serde(default)]
	pub default_editor: Option<String>,

	/// Whether to create secure temp files (Unix only)
	#[serde(default = "default_true")]
	pub secure_temp_files: bool,
}

impl Default for EditorConfig {
	fn default() -> Self { Self { default_editor: None, secure_temp_files: default_true() } }
}
