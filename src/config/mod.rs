mod keymap;

pub use keymap::Keymap;

use crate::command::{Command, CursorMoveType};

impl From<&str> for Command {
    fn from(value: &str) -> Self {
        let command_slice = value.split(" ").collect::<Vec<_>>();

        match command_slice[0] {
            "save"          => Self::Save,
            "quit"          => Self::Quit,
            "change"        => Self::Change,
            "replace_char"  => Self::ReplaceChar,
            "backward_char" => Self::BackwardChar,

            "cancel_mark"   => Self::Mark(true),
            "mark"          => Self::Mark(false),
            "delete_char"   => Self::Delete(true),
            "delete"        => Self::Delete(false),
            "newline"       => Self::NewLine(command_slice[1] == "down"),
            "change_insert" => Self::ChangeInsert(command_slice[1].into()),

            "page_scroll" => {
                let scroll_line = command_slice[1].parse::<isize>()
                    .expect("Invalid argument for command page_scroll!");

                Self::PageScroll(scroll_line)
            },

            "move_cursor" => {
                let within_line = match command_slice[1] {
                    "line" => true,
                    "buffer" => false,
                    _ => panic!("Invalid argument for command move_cursor!")
                };
                let cursor_move: CursorMoveType = command_slice[2].into();

                Self::Move(within_line, cursor_move)
            },

            // TODO: Modify here
            "search" => Self::Save,

            "search_jump" => Self::Save,

            _ => panic!("Invalid command!")
        }
    }
}
