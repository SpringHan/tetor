// Command

use crate::error::AppResult;

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

#[derive(Debug, Clone)]
pub enum Command {
    Move(bool, CursorMoveType),
    SelfInsert(char),
    Modification
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
    // TODO: Fill this func
    pub async fn execute(&self) -> AppResult<()> {
        // match self {
        //     Command::Move(motion_direction) => todo!(),
        //     Command::SelfInsert(_) => todo!(),
        //     Command::Modification => todo!(),
        // }

        Ok(())
    }
}
