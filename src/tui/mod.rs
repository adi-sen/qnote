mod app;
mod editor;
mod markdown;
mod render;

use std::io;

use anyhow::Result;
pub use app::App;
use ratatui::{Terminal, backend::CrosstermBackend, crossterm::{event::DisableMouseCapture, execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode}}};

use crate::{config::Config, db::Database};

pub fn run_tui(db: Database, config: Config) -> Result<()> {
	enable_raw_mode()?;
	let mut stdout = io::stdout();
	execute!(stdout, EnterAlternateScreen)?;
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;

	let mut app = App::new(db, config)?;
	let res = render::run_app(&mut terminal, &mut app);

	disable_raw_mode()?;
	execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
	terminal.show_cursor()?;

	if let Err(err) = res {
		println!("{err:?}");
	}

	Ok(())
}
