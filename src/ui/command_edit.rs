// Command Edit

use crossterm::event::KeyCode;

use crate::{app::App, command::CommandPrior, error::{AppResult, ErrorType}};

#[derive(Debug, PartialEq, Eq)]
pub enum CommandEdit {
    Some(String, usize, CommandPrior),
    None
}

impl CommandEdit {
    pub fn new(init_str: String, cmd: CommandPrior) -> Self {
        let cursor = init_str.len();

        Self::Some(init_str, cursor, cmd)
    }

    /// Return a boolean value. When it's true, means the user have pressed Enter.
    pub fn edit(app: &mut App, key: KeyCode) -> AppResult<bool> {
        let command_edit = &mut app.command_edit;

        if let Self::Some(ref mut content, ref mut cursor, ref cmd) = *command_edit {
            match key {
                KeyCode::Esc => *command_edit = Self::None,

                KeyCode::Left => {
                    if *cursor != 0 {
                        *cursor -= 1;
                    }
                },

                KeyCode::Right => {
                    if *cursor != content.len() {
                        *cursor += 1;
                    }
                },

                KeyCode::Backspace => {
                    if *cursor == 0 {
                        return Ok(false)
                    }

                    content.remove(*cursor - 1);
                    *cursor -= 1;
                },

                KeyCode::Char(_key) => {
                    if *cursor == content.len() {
                        content.push(_key);
                        *cursor += 1;
                        return Ok(false)
                    }

                    content.insert(*cursor, _key);
                    *cursor += 1;
                },

                KeyCode::Enter => {
                    match *cmd {
                        CommandPrior::Search(_) => app.prior_command = CommandPrior::Search(
                            content.to_owned()
                        ),
                        _ => {
                            *command_edit = Self::None;

                            return Err(
                                ErrorType::Specific(
                                    String::from("Unknow command is occuping command line!")
                                ).pack()
                            )
                        }
                    }

                    *command_edit = Self::None;
                    return Ok(true)
                },

                _ => ()
            }
        }

        Ok(false)
    }
}
