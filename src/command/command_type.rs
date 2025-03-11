// Command

use ratatui::crossterm::event::KeyCode;

use crate::{app::App, error::AppResult};

use super::basic::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorMoveType {
    Num(i16),
    Beg,
    End
}

/// The prior command to be executed.
/// When the value of this is not None, apply current key event for current prior command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandPrior {
    None,
    Mark,
    Delete,
    Change,
    Quit(bool),
    ReplaceChar,
    ConfirmError,
    Search(String)
}

#[derive(Debug, Clone)]
pub enum Command {
    Save,
    Quit,
    Mark,
    Change,
    ReplaceChar,
    BackwardChar,
    EscapeCommand,

    Delete(bool),               // Whether delete char
    NewLine(bool),              // Whether open down a new line
    Search(Option<String>),
    SearchJump(bool),           // Whether jump to the next item

    PageScroll(isize),
    Move(bool, CursorMoveType),
    ChangeInsert(CursorMoveType),
}

impl From<&str> for CursorMoveType {
    fn from(value: &str) -> Self {
        match value {
            "^" => Self::Beg,
            "$" => Self::End,
            move_num => {
                let _num = move_num.parse::<i16>()
                    .expect("Error code 1 when parsing &str to i16!");
                Self::Num(_num)
            }
        }
    }
}

impl CursorMoveType {
    /// Return the cursor position after current moving.
    pub async fn after_move(
        self,
        within_line: bool,
        before: (u16, u16),
        file_state: &mut crate::fs::FileState
    ) -> AppResult<(u16, u16)>
    {
        if self == Self::Num(0) {
            return Ok(before)
        }

        let (max, modify_ref);
        let mut after = before;

        if within_line {
            max = file_state.get_lines(before.1, before.1)
                .await?[0]
                .chars()
                .count() - 1;
            modify_ref = &mut after.0;
        } else {
            max = file_state.content_ref().lock().await.len() - 1;
            modify_ref = &mut after.1;
        }

        match self {
            CursorMoveType::Num(i) => {
                let after_move = *modify_ref as i16 + i;

                if after_move < 0 {
                    *modify_ref = 0;
                } else if after_move as usize >= max {
                    *modify_ref = max as u16;
                } else {
                    *modify_ref = after_move as u16;
                }

            },
            CursorMoveType::Beg => {
                *modify_ref = 0;
                if !within_line {
                    after.0 = 0;
                }
            },
            CursorMoveType::End => {
                *modify_ref = max as u16;
                if !within_line {
                    after.0 = 0;
                }
            },
        }

        if !within_line {
            let new_line_length = file_state.get_lines(after.1, after.1)
                .await?[0]
                .chars()
                .count() - 1;

            if after.0 > new_line_length as u16 {
                after.0 = new_line_length as u16;
            }
        }

        Ok(after)
    }
}

impl Command {
    // NOTE: Every command will return a boolean value. When it's true, update the
    // stylized content.
    pub async fn execute(&self, app: &mut App, key: Option<KeyCode>) -> AppResult<bool> {
        Ok(match *self {
            Command::Mark                      => mark(app, key)?,
            Command::Save                      => save(app).await?,
            Command::Quit                      => quit(app, key).await,
            Command::Change                    => change(app, key).await?,
            Command::NewLine(down)             => newline(app, down).await,
            Command::BackwardChar              => backward_char(app).await?,
            Command::EscapeCommand             => escape_command(app).await?,
            Command::ReplaceChar               => replace_char(app, key).await?,
            Command::SearchJump(next)          => search_jump(app, next).await?,
            Command::PageScroll(move_line)     => page_scroll(app, move_line).await,
            Command::Search(ref pattern)       => search(app, pattern.to_owned()).await?,
            Command::ChangeInsert(cursor_move) => change_insert(app, cursor_move).await?,

            Command::Move(within_line, cursor_move) => move_cursor(
                app,
                within_line,
                cursor_move
            ).await?,

            Command::Delete(_delete_char) => {
                if _delete_char {
                    delete_char(app).await?
                } else {
                    delete(app, key).await?
                }
            }
        })
    }
}
