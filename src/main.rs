mod ui;
mod fs;
mod app;
mod error;
mod config;
mod command;

use std::time::Duration;
use std::io::stderr;

use app::{handle_input, App};
use crossterm::event::{self, KeyCode, KeyEventKind};
use error::AppResult;
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

use tokio::runtime::Runtime;

fn main() -> AppResult<()> {
    enable_raw_mode()?;
    execute!(stderr(), EnterAlternateScreen)?;

    // Frame init
    let backend = CrosstermBackend::new(stderr());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let rt = Runtime::new().unwrap();

    rt.block_on(app.init_app())?;

    loop {
        terminal.draw(|frame| ui::main_frame(frame, &mut app))?;

        if app.prior_command == command::CommandPrior::Quit(true) {
            break;
        }

        if poll(Duration::from_millis(200))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match handle_input(&mut app, key.code, &rt) {
                        Ok(_) => (),
                        Err(err) => app.app_errors.append_errors(
                            err.into_iter()
                        ),
                    }
                }
            }
        }
    }

    execute!(stderr(), LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}
