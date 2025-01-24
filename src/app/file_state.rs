// File State

use std::path::{Path, PathBuf};

pub struct FileState {
    path: PathBuf,
    content: String,
}
