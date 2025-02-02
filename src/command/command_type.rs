// Command

use crate::error::AppResult;

#[derive(Debug, Clone, Copy)]
enum MotionDirection {
    Up,
    Down,
    Left,
    Right
}

pub struct ModifyCommand {
}

#[derive(Debug, Clone)]
pub enum Command {
    Move(MotionDirection),
    SelfInsert(char),
    Modification
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
