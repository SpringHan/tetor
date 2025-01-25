mod ui;
mod fs;
mod app;
mod error;
mod config;
mod command;

use std::io::stderr;
use std::error::Error;

use ratatui::{
    Terminal,
    backend::CrosstermBackend
};

use crossterm::{
    execute,
    event::poll,
    terminal::{
        enable_raw_mode,
        disable_raw_mode,
        EnterAlternateScreen,
        LeaveAlternateScreen
    }
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    execute!(stderr(), EnterAlternateScreen)?;

    execute!(stderr(), LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}
