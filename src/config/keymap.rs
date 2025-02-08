// Keymap

use std::{collections::HashMap, path::PathBuf, str::FromStr};

use crossterm::event::KeyCode;
use tokio::io::AsyncReadExt;
use toml_edit::DocumentMut;

use crate::{command::Command, error::AppResult};

/// The keymap for storing normal modal keybindings.
#[derive(Debug, Default)]
pub struct Keymap {
    maps: HashMap<KeyCode, Command>
}

impl Keymap {
    pub async fn init(&mut self) -> AppResult<()> {
        let panic_str = "Wrong content in config file!";
        let document = Self::get_config_doc().await;
        let keymap = document["config"]["keymap"]
            .as_array()
            .expect(panic_str);
        
        for bind in keymap.into_iter() {
            let key_map = bind.as_inline_table()
                .expect(panic_str);

            let key = Self::parse_key(
                key_map.get("key")
                    .expect(panic_str)
                    .as_str()
                    .expect(panic_str)
            );

            let command: Command = key_map.get("run")
                .expect(panic_str)
                .as_str()
                .expect(panic_str)
                .into();

            self.maps.insert(key, command);
        }

        Ok(())
    }

    async fn get_config_doc() -> DocumentMut {
        let user_name = std::env::var("USER")
            .expect("Unable to get current user name!");

        let path_prefix = if &user_name == "root" {
            String::from("/root")
        } else {
            format!("/home/{}", user_name)
        };

        let config_path = PathBuf::from(
            format!("{}/.config/springhan/tetor/config.toml", path_prefix)
        );

        let mut doc_str = String::new();
        let mut file = tokio::fs::File::open(config_path)
            .await
            .expect("Cannot find config.toml file!");

        file.read_to_string(&mut doc_str)
            .await
            .expect("Failed to read config.toml file!");

        DocumentMut::from_str(&doc_str)
            .expect("Failed to parse config toml!")
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
            },
            _ => panic!("Invalid key for parsing!")
        }
    }
}
