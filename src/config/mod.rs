mod keymap;

use crate::command::{Command, CursorMoveType};

pub use keymap::Keymap;

impl From<&str> for Command {
    fn from(value: &str) -> Self {
        let command_slice = value.split(" ").collect::<Vec<_>>();

        match command_slice[0] {
            "save"           => Self::Save,
            "quit"           => Self::Quit,
            "change"         => Self::Change,
            "replace_char"   => Self::ReplaceChar,
            "backward_char"  => Self::BackwardChar,
            "escape_command" => Self::EscapeCommand,

            "cancel_mark"    => Self::Mark(true),
            "mark"           => Self::Mark(false),
            "delete_char"    => Self::Delete(true),
            "search"         => Self::Search(None),
            "delete"         => Self::Delete(false),
            "newline"        => Self::NewLine(command_slice[1] == "down"),
            "change_insert"  => Self::ChangeInsert(command_slice[1].into()),
            "search_jump"    => Self::SearchJump(command_slice[1] == "next"),

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

            _ => panic!("Invalid command!")
        }
    }
}
