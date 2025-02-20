mod keymap;
mod option;

use std::{path::PathBuf, str::FromStr};

use tokio::io::AsyncReadExt;
use toml_edit::DocumentMut;

use crate::error::AppResult;

pub use keymap::Keymap;
pub use option::*;

#[macro_export]
macro_rules! config_throw_error {
    ($x: expr, $y: expr) => {
        match $x {
            Some(result) => result,
            None => return Err(ErrorType::Specific(String::from($y)).pack())
        }
    };
}

pub(crate) async fn init_config(keymap: &mut Keymap, options: &mut AppOption) -> AppResult<()> {
    let document = get_config_doc().await;
    keymap.init(&document)?;
    options.init(&document)?;

    Ok(())
}

pub(self) async fn get_config_doc() -> DocumentMut {
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
