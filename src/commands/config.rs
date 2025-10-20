use anyhow::Result;

use crate::{config::Config, utils::confirm};

/// Handles the config command - generates or shows configuration
pub fn handle_config(show: bool) -> Result<()> {
	if show {
		// Show current configuration
		let config = Config::load()?;
		let config_str = toml::to_string_pretty(&config)?;
		println!("Current configuration:\n");
		println!("{config_str}");
		let config_path = Config::get_config_path()?;
		println!("\nConfig file location: {}", config_path.display());
	} else {
		// Generate default configuration file
		let config_path = Config::get_config_path()?;

		if config_path.exists() {
			println!("Config file already exists at: {}", config_path.display());
			if !confirm("Overwrite existing config?") {
				println!("Cancelled.");
				return Ok(());
			}
		}

		let default_config = Config::default();
		default_config.save()?;
		println!("Generated default configuration file at: {}", config_path.display());
		println!("\nYou can edit this file to customize qnote's behavior.");
	}
	Ok(())
}
