// App

mod error;
mod file_state;
mod modal;
mod keymap;

pub use error::{AppResult, AppError};

use std::path::{Path, PathBuf};

pub struct App {
    file_path: Option<PathBuf>,
}

impl App {
    
}
