// Keymap

use std::collections::HashMap;

use toml_edit::DocumentMut;
use ratatui::crossterm::event::KeyCode;

use crate::{
    command::{Command, CursorMoveType},
    error::{AppResult, ErrorType},
    config_throw_error,
};

/// The keymap for storing normal modal keybindings.
#[derive(Debug, Default)]
pub struct Keymap {
    maps: HashMap<KeyCode, Command>
}

impl Keymap {
    pub fn keymap(&self) -> &HashMap<KeyCode, Command> {
        &self.maps
    }

    pub fn init(&mut self, document: &DocumentMut) -> AppResult<()> {
        let panic_str = "Wrong format for keymap in config file!";
        let keymap = config_throw_error!(
            document["config"]["keymap"].as_array(),
            panic_str
        );
        
        for bind in keymap.iter() {
            let key_map = config_throw_error!(
                bind.as_inline_table(),
                panic_str
            );

            let key = Self::parse_key(
                config_throw_error!(
                    config_throw_error!(key_map.get("key"), panic_str).as_str(),
                    panic_str
                )
            );

            let command: Command = config_throw_error!(
                config_throw_error!(key_map.get("run"), panic_str).as_str(),
                panic_str
            ).into();

            self.maps.insert(key, command);
        }

        Ok(())
    }

    fn parse_key(key: &str) -> KeyCode {
        match key {
            "Up" => KeyCode::Up,
            "Left" => KeyCode::Left,
            "Down" => KeyCode::Down,
            "Right" => KeyCode::Right,

            "Tab" => KeyCode::Tab,
            "ESC" => KeyCode::Esc,
            "Enter" => KeyCode::Enter,
            "Backspace" => KeyCode::Backspace,

            key => {
                let byte = key.as_bytes()[0];
                if byte < 32 || byte > 126 {
                    panic!("Invalid key for parsing!")
                }

                KeyCode::Char(byte as char)
            }
        }
    }
}

impl From<&str> for Command {
    fn from(value: &str) -> Self {
        let command_slice = value.split(" ").collect::<Vec<_>>();

        match command_slice[0] {
            "save"           => Self::Save,
            "mark"           => Self::Mark,
            "quit"           => Self::Quit,
            "change"         => Self::Change,
            "replace_char"   => Self::ReplaceChar,
            "backward_char"  => Self::BackwardChar,
            "escape_command" => Self::EscapeCommand,

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
