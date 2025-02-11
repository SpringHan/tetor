// Handle Input

use crossterm::event::KeyCode;
use tokio::runtime::Runtime;

use crate::{
    command::{backward_char, insert_char, Command, CommandPrior},
    error::{AppResult, ErrorType},
    ui::ModalType
};

use super::App;

pub fn handle_input(app: &mut App, key: KeyCode, rt: &Runtime) -> AppResult<()> {
    if app.prior_command == CommandPrior::ConfirmError {
        app.prior_command = CommandPrior::None;
        app.app_errors.throw();

        return Ok(())
    }

    if app.get_modal().modal() == ModalType::Insert {
        match key {
            KeyCode::Char(_key) => rt.block_on(insert_char(app, _key))?,
            KeyCode::Esc => app.get_modal().switch_normal(),
            KeyCode::Backspace => rt.block_on(backward_char(app))?,
            _ => (),
        }

        return Ok(())
    }

    rt.block_on(async {
        let prior_command = match app.prior_command {
            CommandPrior::Mark => Some(Command::Mark(false)),
            CommandPrior::Delete => Some(Command::Delete(false)),
            CommandPrior::Change => Some(Command::Change),
            CommandPrior::ReplaceChar => Some(Command::ReplaceChar),
            CommandPrior::Quit(_) => Some(Command::Quit),
            CommandPrior::ConfirmError => panic!("Unknow error!"),
            CommandPrior::None => None,
        };

        if let Some(command) = prior_command {
            command.execute(app, Some(key)).await?;

            return Ok(())
        }

        if let Some(command) = app.get_command(key) {
            command.execute(app, None).await?;

            return Ok(())
        }

        Err(
            ErrorType::Specific(
                String::from("Invalid key command")
            ).pack()
        )
    })?;

    Ok(())
}
