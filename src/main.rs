mod ui;
mod fs;
mod app;
mod error;
mod config;
mod command;

use std::time::Duration;
use std::io::stderr;
use std::error::Error;

use app::App;
use crossterm::event::{self, KeyCode, KeyEventKind};
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

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    execute!(stderr(), EnterAlternateScreen)?;

    // Frame init
    let backend = CrosstermBackend::new(stderr());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let rt = Runtime::new().unwrap();
    // rt.block_on(app.init_file())
    rt.block_on(async {
        app.init_file().await.expect("The editor cannot open this file!");
    });

    loop {
        terminal.draw(|frame| ui::main_frame(frame, &mut app))?;

        if poll(Duration::from_millis(200))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if key.code == KeyCode::Char('q') {
                        break;
                    }

                    if key.code == KeyCode::Down {
                        app.editor_state.scroll_down(1);
                    }
                }
            }
        }
    }

    execute!(stderr(), LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}
