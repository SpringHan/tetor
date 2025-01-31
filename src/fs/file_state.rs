// File State

use super::type_convert::StyleConvert;
use crate::error::{self, AppResult, AppError};

use ratatui::style::Style;
use tokio::fs;
use tokio::sync::{mpsc, Mutex};
use tokio::io::AsyncReadExt;

use syntect::{
    parsing::SyntaxSet,
    easy::HighlightLines,
    util::LinesWithEndings,
    highlighting::{Theme, ThemeSet},
};

use std::sync::Arc;
use std::path::{Path, PathBuf};

pub type LineVec = Vec<ContentLine>;
type StylizedContent = Vec<(ratatui::style::Style, String)>;

/// A structure storing single line of stylized content.
#[derive(Debug)]
pub struct ContentLine(StylizedContent);

#[derive(Debug)]
pub struct FileState {
    path: PathBuf,
    content: Arc<Mutex<LineVec>>,
    theme: Arc<Theme>,
    syntax_set: Arc<SyntaxSet>
}

impl ContentLine {
    pub fn get_iter<'a>(&'a self) -> impl Iterator<Item = &'a (Style, String)> {
        self.0.iter()
    }
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
        self.path = path.as_ref().to_path_buf();

        // TODO: Return the parse result
        Ok(())
    }

    pub async fn reset(&mut self) {
        self.path = PathBuf::default();
        self.content.lock().await.clear();
    }

    async fn parse_content<P>(
        &self,
        path: P,
        mut rx: mpsc::UnboundedReceiver<String>
    ) -> AppResult<LineVec>
    where P: AsRef<Path>
    {
        let mut result: LineVec = Vec::new();
        let find_syntax = self.syntax_set.find_syntax_for_file(
            path.as_ref()
        )?;
        if find_syntax.is_none() {
            return Err(
                error::ErrorType::Specific(String::from(
                    "Cannot find syntax for current file!"
                )).pack()
            )
        }

        let syntax = find_syntax.unwrap();
        let mut h = HighlightLines::new(syntax, &self.theme);

        if let Some(content) = rx.recv().await {
            for line in LinesWithEndings::from(&content) {
                // TODO: Return no highlight content
                let ranges = h.highlight_line(line, &self.syntax_set)
                    .unwrap();

                result.push(ContentLine(
                    ranges.into_iter()
                        .map(|(style, _content)| (style.to_rstyle(), String::from(_content)))
                        .collect::<StylizedContent>()
                ));
            }
        }

        Ok(result)
    }
}

impl Default for FileState {
    fn default() -> Self {
        FileState {
            path: PathBuf::default(),
            content: Arc::new(Mutex::new(Vec::new())),
            theme: Arc::new(
                ThemeSet::load_defaults().themes["base16-ocean.dark"].to_owned()
            ),
            syntax_set: Arc::new(SyntaxSet::load_defaults_newlines())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_test() {
        let mut runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let mut file_state = FileState::default();
            file_state.init(PathBuf::from("/home/spring/test.el")).await?;

            println!("{:?}", file_state.content);
            file_state.reset().await;

            Ok::<(), AppError>(())
        }).unwrap();
    }
}
