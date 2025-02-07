mod keymap;

pub use keymap::Keymap;

use crate::command::Command;

impl From<String> for Command {
    fn from(value: String) -> Self {
        todo!()
    }
}
