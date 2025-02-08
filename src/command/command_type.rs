// Command

use crossterm::event::KeyCode;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandPrior {
    Mark,
    Delete,
    Change,
    Kmacro,
    ReplaceChar,
    Quit(bool),
    None
}

// TODO: Add search command
#[derive(Debug, Clone)]
pub enum Command {
    Save,
    Quit,
    Change,
    ReplaceChar,

    Mark(bool),                 // Whether cancel mark
    Delete(bool),               // Whether delete char
    NewLine(bool),              // Whether open down a new line

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

        let (length, modify_ref);
        let mut after = before;

        if within_line {
            length = file_state.get_lines(before.1, before.1).await?[0].len() - 1;
            modify_ref = &mut after.0;
        } else {
            length = file_state.content_ref().lock().await.len() - 1;
            modify_ref = &mut after.1;
        }

        match self {
            CursorMoveType::Num(i) => {
                let after_move = before.0 as i16 + i;

                if after_move < 0 {
                    *modify_ref = 0;

                    return Ok(after)
                }

                if after_move as usize >= length {
                    *modify_ref = length as u16;

                    return Ok(after)
                }

                *modify_ref = after_move as u16;
            },
            CursorMoveType::Beg => *modify_ref = 0,
            CursorMoveType::End => *modify_ref = length as u16,
        }

        Ok(after)
    }
}

impl Command {
    pub async fn execute(&self, app: &mut App, key: Option<KeyCode>) -> AppResult<()> {
        match *self {
            Command::Save                      => save(app).await?,
            Command::Quit                      => quit(app, key).await,
            Command::Change                    => change(app, key).await?,
            Command::NewLine(down)             => newline(app, *down).await,
            Command::ReplaceChar               => replace_char(app, key).await?,
            Command::PageScroll(move_line)     => page_scroll(app, *move_line).await,
            Command::ChangeInsert(cursor_move) => change_insert(app, *cursor_move).await?,

            Command::Move(within_line, cursor_move) => move_cursor(
                app,
                within_line,
                cursor_move
            ).await?,

            Command::Mark(_cancel_mark) => {
                if _cancel_mark {
                    cancel_mark(app);
                } else {
                    mark(app, key)?
                }
            },

            Command::Delete(_delete_char) => {
                if _delete_char {
                    delete_char(app).await?;
                } else {
                    delete(app, key).await?;
                }
            }
        }

        Ok(())
    }
}
