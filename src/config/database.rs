use serde::{Deserialize, Serialize};

use super::defaults::default_true;

/// Database configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
	/// Enable Write-Ahead Logging for better performance (disable for network
	/// drives)
	#[serde(default = "default_true")]
	pub wal_mode: bool,

	/// Database cache size in kilobytes (negative value = KB, positive = pages)
	#[serde(default = "default_cache_size_kb")]
	pub cache_size_kb: i32,

	/// Synchronous mode: OFF, NORMAL, FULL, or EXTRA
	#[serde(default = "default_synchronous")]
	pub synchronous: String,

	/// Temp store: DEFAULT, FILE, or MEMORY
	#[serde(default = "default_temp_store")]
	pub temp_store: String,
}

const fn default_cache_size_kb() -> i32 {
	-64000 // 64MB cache (negative = KB)
}

fn default_synchronous() -> String { "NORMAL".to_string() }

fn default_temp_store() -> String { "MEMORY".to_string() }

impl Default for DatabaseConfig {
	fn default() -> Self {
		Self {
			wal_mode:      default_true(),
			cache_size_kb: default_cache_size_kb(),
			synchronous:   default_synchronous(),
			temp_store:    default_temp_store(),
		}
	}
}
