// App

mod keymap;
mod handle_input;

pub use handle_input::handle_input;
use tokio::sync::Mutex;

use crate::{error::{AppError, AppResult}, fs::{FileState, LineVec}, ui::{Editor, EditorState}};

use std::{path::{Path, PathBuf}, sync::Arc};

#[derive(Debug)]
pub struct App {
    file_state: FileState,

    keymap: keymap::Keymap,

    pub(crate) editor_state: EditorState,

    app_errors: AppError,
}

impl App {
    pub fn new() -> Self {
        App {
            file_state: FileState::default(),
            keymap: keymap::Keymap::init(),
            editor_state: EditorState::default(),
            app_errors: AppError::default(),
        }
    }

    pub async fn init_file(&mut self) -> AppResult<()> {
        self.file_state.init("/home/spring/test.el").await?;

        Ok(())
    }

    pub fn get_content(&self) -> Arc<Mutex<LineVec>> {
        self.file_state.content()
    }
}
