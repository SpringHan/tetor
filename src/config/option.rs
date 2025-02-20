// App Option

use toml_edit::DocumentMut;

use crate::{config_throw_error, error::{AppResult, ErrorType}};

#[derive(Debug, Clone)]
pub struct AppOption {
    pub tab_indent: bool
}

impl Default for AppOption {
    fn default() -> Self {
        Self {
            tab_indent: false
        }
    }
}

impl AppOption {
    pub fn init(&mut self, document: &DocumentMut) -> AppResult<()> {
        let panic_str = "Wrong format for App Options in config file!";
        let options = document["config"].get("options");

        if options.is_none() {
            return Ok(())
        }

        let options = config_throw_error!(
            options.unwrap().as_table(),
            panic_str
        );

        for (prop, value) in options.iter() {
            match prop {
                "tab_indent" => self.tab_indent = config_throw_error!(
                    value.as_bool(),
                    panic_str
                ),
                _ => return Err(
                    ErrorType::Specific(
                        format!("Unknow option: {}", prop)
                    ).pack()
                )
            }
        }

        Ok(())
    }
}
