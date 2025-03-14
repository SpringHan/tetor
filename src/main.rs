mod ui;
mod fs;
mod app;
mod utils;
mod error;
mod config;
mod command;

use std::time::Duration;
use std::io::stderr;

use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    crossterm::{
        execute,
        event::poll,
        cursor::{Hide, Show},
        event::{self, KeyEventKind},
        terminal::{
            enable_raw_mode,
            disable_raw_mode,
            EnterAlternateScreen,
            LeaveAlternateScreen
        }
    }
};

use tokio::runtime::Runtime;

use app::{handle_input, App};
use error::{AppResult, ErrorType};

fn main() -> AppResult<()> {
    // Frame init
    let backend = CrosstermBackend::new(stderr());
    let mut terminal = Terminal::new(backend)?;

    let mut args: Vec<String> = std::env::args().collect();
    handle_cli_args(&mut args)?;

    let mut app = App::new();
    let rt = Runtime::new().unwrap();

    rt.block_on(app.init_app(args[1].to_owned()))?;

    enable_raw_mode()?;
    execute!(stderr(), EnterAlternateScreen, Hide)?;

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

    execute!(stderr(), LeaveAlternateScreen, Show)?;
    disable_raw_mode()?;

    Ok(())
}

// TODO: Is this necessary?
fn handle_cli_args(args: &mut Vec<String>) -> AppResult<()> {
    let err = ErrorType::Specific(
        String::from("Wrong arguments for this app!")
    ).pack();

    if args.len() < 2 {
        return Err(err)
    }

    loop {
        if args.len() == 0 {
            return Err(err)
        }

        if args[0] == "tetor" {
            break;
        }

        args.remove(0);
    }

    Ok(())
}
