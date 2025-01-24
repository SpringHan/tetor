// File State

use super::AppResult;

use tokio::fs;
use tokio::io::AsyncReadExt;
use syntect::easy::HighlightLines;

use std::path::{Path, PathBuf};

pub struct FileState {
    path: PathBuf,
    content: String,
}

impl FileState {
    pub async fn init<P: AsRef<Path>>(&mut self, path: P) -> AppResult<()> {
        let mut file = fs::File::open(path.as_ref().to_owned()).await?;
        let mut _content = String::new();

        file.read_to_string(&mut _content).await?;

        self.path = path.as_ref().to_path_buf();
        self.content = _content;

        Ok(())
    }

    pub fn reset(&mut self) {
        self.path = PathBuf::default();
        self.content = String::new();
    }
}

impl Default for FileState {
    fn default() -> Self {
        FileState { path: PathBuf::default(), content: String::new() }
    }
}
