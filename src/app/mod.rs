// App

mod keymap;
mod handle_input;

pub use handle_input::handle_input;
use tokio::sync::Mutex;

use crate::{command::CommandPrior, error::{AppError, AppResult}, fs::{FileState, LineVec}, ui::{Editor, EditorState}};

use std::{path::{Path, PathBuf}, sync::Arc};

#[derive(Debug)]
pub struct App {
    pub file_state: FileState,

    pub editor_state: EditorState,

    pub prior_command: CommandPrior,

    keymap: keymap::Keymap,

    app_errors: AppError,
}

impl App {
    pub fn new() -> Self {
        App {
            file_state: FileState::default(),
            keymap: keymap::Keymap::init(),
            editor_state: EditorState::default(),
            app_errors: AppError::default(),
            prior_command: CommandPrior::None
        }
    }

    // TODO: Modify to add file path as a parameter
    pub async fn init_file(&mut self) -> AppResult<()> {
        self.file_state.init("/home/spring/Rust/hire/src/ui.rs").await?;
        // self.file_state.init("/home/spring/test.el").await?;

        Ok(())
    }

    pub fn get_bg(&self) -> ratatui::style::Color {
        self.file_state.background_color
    }

    pub fn get_modal(&mut self) -> &mut crate::ui::Modal {
        &mut self.editor_state.modal
    }
}
