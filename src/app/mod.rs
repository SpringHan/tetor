// App

mod handle_input;

pub use handle_input::handle_input;
use tokio::sync::Mutex;

use crate::{
    command::{Command, CommandPrior},
    config::Keymap,
    error::{AppError, AppResult},
    fs::{FileState, LineVec},
    ui::{Editor, EditorState}
};

use std::{path::{Path, PathBuf}, sync::Arc};

#[derive(Debug)]
pub struct App {
    pub file_state: FileState,

    pub editor_state: EditorState,

    pub prior_command: CommandPrior,

    keymap: Keymap,

    pub app_errors: AppError,
}

impl App {
    pub fn new() -> Self {
        App {
            file_state: FileState::default(),
            keymap: Keymap::default(),
            editor_state: EditorState::default(),
            app_errors: AppError::default(),
            prior_command: CommandPrior::None
        }
    }

    pub fn get_bg(&self) -> ratatui::style::Color {
        self.file_state.background_color
    }

    pub fn get_modal(&mut self) -> &mut crate::ui::Modal {
        &mut self.editor_state.modal
    }

    pub fn get_command(&self, key: crossterm::event::KeyCode) -> Option<Command> {
        self.keymap.keymap().get(&key).cloned()
    }

    pub async fn init_app(&mut self) -> AppResult<()> {
        let (file_result, keymap_result) = tokio::join!(
            self.file_state.init("/home/spring/Rust/hire/src/ui.rs"),
            self.keymap.init()
        );

        (file_result?, keymap_result?);

        Ok(())
    }
}
