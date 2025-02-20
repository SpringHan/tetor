mod ui;
mod fs;
mod app;
mod utils;
mod error;
mod config;
mod command;

use std::time::Duration;
use std::io::stderr;

use crossterm::event::{self, KeyEventKind};
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

use app::{handle_input, App};
use error::{AppResult, ErrorType};

fn main() -> AppResult<()> {
    // Frame init
    let backend = CrosstermBackend::new(stderr());
    let mut terminal = Terminal::new(backend)?;

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        return Err(
            ErrorType::Specific(
                String::from("Wrong arguments for this app!")
            ).pack()
        )
    }

    let mut app = App::new();
    let rt = Runtime::new().unwrap();

    rt.block_on(app.init_app(args[1].to_owned()))?;

    enable_raw_mode()?;
    execute!(stderr(), EnterAlternateScreen)?;

    loop {
        terminal.draw(|frame| {
            match ui::main_frame(frame, &mut app, &rt) {
                Ok(_) => (),
                Err(err) => {
                    execute!(stderr(), LeaveAlternateScreen).unwrap();
                    disable_raw_mode().unwrap();

                    println!("{}", err.to_string());
                    panic!("See the error generated above caused by rendering frame.")
                },
            }
        })?;

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
