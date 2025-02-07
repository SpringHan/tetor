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
        let document = Self::get_config_doc().await;
        let keymap = document["config"]["keymap"]
            .as_array()
            .expect("Wrong content in config file!");
        
        for bind in keymap.into_iter() {
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

}
