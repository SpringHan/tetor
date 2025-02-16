// App

mod handle_input;

pub use handle_input::handle_input;
use tokio::sync::Mutex;

use crate::{
    command::{Command, CommandPrior},
    config::Keymap,
    error::{AppError, AppResult, ErrorType},
    fs::{FileState, LineVec},
    ui::{Editor, EditorState}
};

#[derive(Debug)]
pub struct App {
    pub file_state: FileState,

    pub editor_state: EditorState,

    pub prior_command: CommandPrior,

    keymap: Keymap,

    pub app_errors: AppError,

    pub ask_msg: Option<String>,
    pub update_stylized: bool,
}

impl App {
    pub fn new() -> Self {
        App {
            file_state: FileState::default(),
            keymap: Keymap::default(),
            editor_state: EditorState::default(),
            app_errors: AppError::default(),
            prior_command: CommandPrior::None,
            ask_msg: None,
            update_stylized: true
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

    pub fn get_modal(&mut self) -> &mut crate::ui::Modal {
        &mut self.editor_state.modal
    }

    pub fn get_command(&self, key: crossterm::event::KeyCode) -> Option<Command> {
        self.keymap.keymap().get(&key).cloned()
    }

    pub async fn init_app(&mut self) -> AppResult<()> {
        // TODO: If cannot find parse, use basic color for all the fonts.
        let (file_result, keymap_result) = tokio::join!(
            // self.file_state.init("/home/spring/test.el"),
            self.file_state.init("/home/spring/Rust/hire/src/ui.rs"),
            // self.file_state.init("/var/log/pacman.log"),
            // self.file_state.init("/home/spring/.config/hypr/hyprland.conf"),
            self.keymap.init()
        );

        (file_result?, keymap_result?);

        Ok(())
    }
}
