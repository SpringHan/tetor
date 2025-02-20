// App

mod handle_input;
mod search;

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    command::{Command, CommandPrior},
    config::{init_config, AppOption, Keymap},
    error::{AppError, AppResult, ErrorType},
    fs::FileState,
    ui::{CommandEdit, EditorState}
};

pub use handle_input::handle_input;
pub use search::SearchIndicates;

#[derive(Debug)]
pub struct App {
    keymap: Keymap,
    options: AppOption,
    search_result: Arc<Mutex<SearchIndicates>>,

    pub file_state: FileState,

    pub editor_state: EditorState,

    pub prior_command: CommandPrior,

    pub app_errors: AppError,

    pub ask_msg: Option<String>,
    pub command_edit: CommandEdit,

    pub update_stylized: bool,
}

impl App {
    pub fn new() -> Self {
        App {
            file_state: FileState::default(),
            keymap: Keymap::default(),
            options: AppOption::default(),
            editor_state: EditorState::default(),
            app_errors: AppError::default(),
            prior_command: CommandPrior::None,
            ask_msg: None,
            update_stylized: true,
            command_edit: CommandEdit::None,
            search_result: Arc::new(Mutex::new(
                SearchIndicates::default()
            ))
        }
    }

    pub fn get_bg(&self) -> AppResult<ratatui::style::Color> {
        if let Some(color) = self.file_state.background_color {
            return Ok(color)
        }

        Err(
            ErrorType::Specific(
                String::from("Failed to get background color!")
            ).pack()
        )
    }

    // Get reference part starts from here
    pub fn get_modal(&mut self) -> &mut crate::ui::Modal {
        &mut self.editor_state.modal
    }

    pub fn get_command(&self, key: crossterm::event::KeyCode) -> Option<Command> {
        self.keymap.keymap().get(&key).cloned()
    }

    pub fn options(&self) -> &AppOption {
        &self.options
    }

    pub fn search_ref(&self) -> &Arc<Mutex<SearchIndicates>> {
        &self.search_result
    }

    // Initialization part starts from here
    pub async fn init_app(&mut self, path: String) -> AppResult<()> {
        let (file_result, keymap_result) = tokio::join!(
            self.file_state.init(path),
            // self.file_state.init("/home/spring/test.el"),
            // self.file_state.init("/home/spring/Rust/hire/src/ui.rs"),
            // self.file_state.init("/var/log/pacman.log"),
            // self.file_state.init("/home/spring/.config/hypr/hyprland.conf"),
            init_config(&mut self.keymap, &mut self.options)
        );

        (file_result?, keymap_result?);

        self.editor_state.update_linenr(
            self.file_state.content_ref().lock().await.len()
        );

        Ok(())
    }
}
