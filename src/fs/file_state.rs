// File State

use crate::error::{self, AppResult, AppError};

use tokio::fs;
use tokio::sync::mpsc;
use tokio::io::AsyncReadExt;

use ratatui::style::{
    Style as RStyle,
    Color as RColor
};

use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{
    ThemeSet,
    Theme,
    Style as HStyle,
    Color as HColor
};

use std::sync::Arc;
use std::path::{Path, PathBuf};

pub struct FileState {
    path: PathBuf,
    content: Vec<(RStyle, String)>,
    theme: Arc<Theme>,
    syntax_set: Arc<SyntaxSet>
}

impl FileState {
    pub async fn init<P: AsRef<Path>>(&mut self, path: P) -> AppResult<()> {
        let mut file = fs::File::open(path.as_ref().to_owned()).await?;
        let (tx, rx) = mpsc::unbounded_channel();

        let read_task = async {
            let mut _content = String::new();

            file.read_to_string(&mut _content).await?;
            tx.send(_content)
                .expect("Error Code 0 in file_state.rs: Sender cannot send msg!");

            Ok::<(), AppError>(())
        };

        let (read_result, parse_result) = tokio::join!(
            read_task,
            self.parse_content(path.as_ref().to_owned(), rx)
        );

        read_result?;
        parse_result?;

        Ok(())
    }

    async fn parse_content<P>(
        &self,
        path: P,
        mut rx: mpsc::UnboundedReceiver<String>
    ) -> AppResult<()>
    where P: AsRef<Path>
    {
        while let Some(content) = rx.recv().await {
            let find_syntax = self.syntax_set.find_syntax_by_path(
                &path.as_ref().to_string_lossy()
            );

            if find_syntax.is_none() {
                return Err(
                    error::ErrorType::Specific(String::from(
                        "Cannot find syntax for current file!"
                    )).pack()
                )
            }

            let syntax = find_syntax.unwrap();
        }

        // TODO: Return parsed vec
        Ok(())
    }

    pub fn reset(&mut self) {
        self.path = PathBuf::default();
        self.content.clear();
    }
}

impl Default for FileState {
    fn default() -> Self {
        FileState {
            path: PathBuf::default(),
            content: Vec::new(),
            theme: Arc::new(
                ThemeSet::load_defaults().themes["base16-ocean.dark"].to_owned()
            ),
            syntax_set: Arc::new(SyntaxSet::load_defaults_newlines())
        }
    }
}
