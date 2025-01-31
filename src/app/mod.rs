// App

mod keymap;

use crate::{error::AppResult, fs::FileState, ui::{Editor, EditorState}};

use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct App {
    file_state: FileState,

    keymap: keymap::Keymap,

    // Editor
    editor: Editor,
    editor_state: EditorState
}

impl App {
    
}
