// File State

use super::type_convert::{ColorConvert, StyleConvert};
use crate::error::{AppError, AppResult, ErrorType};

use ratatui::style::{Color, Style};
use tokio::{fs, sync::Mutex};
use tokio::sync::mpsc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use syntect::{
    parsing::SyntaxSet,
    easy::HighlightLines,
    highlighting::{Theme, ThemeSet},
};

use std::sync::Arc;
use std::path::{Path, PathBuf};

pub type StylizedVec = Vec<ContentLine>;
type LineVec = Vec<String>;
type StylizedContent = Vec<(ratatui::style::Style, String)>;

/// A structure storing single line of stylized content.
#[derive(Debug, Clone)]
pub struct ContentLine(StylizedContent);

// TODO: Do not load all the file when the file is too large
#[derive(Debug)]
pub struct FileState {
    pub background_color: Option<Color>,
    content: Arc<Mutex<LineVec>>,
    stylized: Arc<Mutex<StylizedVec>>,

    file_modified: Arc<Mutex<bool>>,

    path: PathBuf,
    theme: Theme,
    syntax_set: SyntaxSet
}

impl Into<String> for ContentLine {
    fn into(self) -> String {
        self.0.into_iter()
            .map(|(_, span)| span)
            .collect::<Vec<_>>()
            .join("")
    }
}

impl ContentLine {
    pub fn get_iter<'a>(&'a self) -> impl Iterator<Item = &'a (Style, String)> {
        self.0.iter()
    }
}

// Main Implementation
impl FileState {
    pub async fn file_modify(&self) {
        *self.file_modified.lock().await = true;
    }

    pub async fn not_save(&self) -> bool {
        *self.file_modified.lock().await
    }

    pub fn content_ref(&self) -> &Arc<Mutex<LineVec>> {
        &self.content
    }

    pub fn stylized_ref(&self) -> &Arc<Mutex<StylizedVec>> {
        &self.stylized
    }

    pub async fn init(&mut self, path: String) -> AppResult<()> {
        let path = Self::get_absolute(path);

        let file = fs::File::open(path.to_owned()).await?;
        let content_ref = Arc::clone(&self.content);

        let read_result = tokio::join!(async move {
            let mut content = content_ref.lock().await;
            let mut reader_lines = BufReader::new(file).lines();

            while let Some(mut line) = reader_lines.next_line().await? {
                // line = strip_ansi_escapes::strip_str(&line);
                line.push('\n');
                content.push(line.to_owned());
            }

            Ok::<(), AppError>(())
        });

        read_result.0?;
        self.path = path;

        Ok(())
    }

    pub async fn refresh_stylized(
        &mut self,
        start: usize,
        height: usize,
    ) -> AppResult<()> {
        let content = self.content.lock().await;

        let end = if start + height > content.len() {
            content.len()
        } else {
            start + height
        };

        // Parse content
        let (tx, rx) = mpsc::unbounded_channel();
        let sender_task = async move {
            for line in content[start..end].iter() {
                if tx.is_closed() {
                    return Err(
                        ErrorType::Specific(
                            String::from("Channel closed when initializing file information")
                        ).pack()
                    )
                }

                tx.send(line.to_owned()).unwrap();
            }

            Ok::<(), AppError>(())
        };

        let (sender_result, parse_result) = tokio::join!(
            sender_task,
            self.parse_content(self.path.to_owned(), rx)
        );

        sender_result?;
        let parse_result = parse_result?;

        // Update variables
        if self.background_color.is_none() {
            self.background_color = Some(parse_result.1);
        }

        let mut stylized = self.stylized.lock().await;
        stylized.clear();
        stylized.extend(parse_result.0.into_iter());

        Ok(())
    }

    // TODO: Use string as parameter rather than channel when there's no need
    async fn parse_content<'a, P>(
        &self,
        path: P,
        mut rx: mpsc::UnboundedReceiver<String>,
    ) -> AppResult<(StylizedVec, Color)>
    where P: AsRef<Path>
    {
        let find_syntax = self.syntax_set.find_syntax_for_file(
            path.as_ref()
        )?;

        let mut result: StylizedVec = Vec::new();
        let mut get_bg = false;
        let mut background_color: Color = Color::default();

        let mut h = if find_syntax.is_some() {
            Some(HighlightLines::new(find_syntax.unwrap(), &self.theme))
        } else {
            None
        };

        while let Some(content) = rx.recv().await {
            // Highligth line
            if let Some(ref mut _h) = h {
                let ranges = _h.highlight_line(&content, &self.syntax_set)
                    .unwrap();

                if !get_bg {
                    get_bg = true;

                    background_color = ranges.get(0)
                        .expect("Error code 1 at parse_content in file_state.rs")
                        .0
                        .background
                        .to_rcolor();
                }

                result.push(ContentLine(
                    ranges.into_iter()
                        .map(|(style, _content)| (style.to_rstyle(), String::from(_content)))
                        .collect::<StylizedContent>()
                ));

                continue;
            }

            // Use default color
            if !get_bg {
                background_color = Color::Black;
                get_bg = true;
            }

            result.push(ContentLine(
                vec![(Style::default(), content.to_owned())]
            ));
            
        }

        Ok((result, background_color))
    }

    /// Get lines from file content with range.
    pub async fn get_lines(&self, from: u16, to: u16) -> AppResult<Vec<String>> {
        let (from, to) = (from as usize, to as usize);
        let file_lines = self.content.lock().await;

        if file_lines.is_empty() {
            return Err(
                ErrorType::Specific(
                    String::from("Cannot execute current editing operation as it's a empty file!")
                ).pack()
            )
        }

        if from > to || to >= file_lines.len() {
            return Err(
                ErrorType::Specific(
                    String::from("Attempt to get lines with wrong range.")
                ).pack()
            )
        }

        Ok(file_lines[from..=to].to_owned())
    }

    /// Modify lines with modified lines & range.
    /// Update its syntax highlight in the meanwhile.
    pub async fn modify_lines(
        &mut self,
        from: u16,
        to: u16,
        lines: Vec<String>
    ) -> AppResult<()>
    {
        let (from, to) = (from as usize, to as usize);
        let mut file_lines = self.content.lock().await;

        if from > to || to >= file_lines.len() {
            return Err(
                ErrorType::Specific(
                    String::from("Attempt to modify lines with wrong range.")
                ).pack()
            )
        }

        file_lines.splice(from..=to, lines);

        self.file_modify().await;

        Ok(())
    }

    pub async fn save_content(&mut self) -> AppResult<()> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(self.path.to_owned()).await?;

        for line in self.content.lock().await.iter() {
            file.write(line.as_bytes()).await?;
        }

        *self.file_modified.lock().await = false;

        Ok(())
    }

    fn get_absolute(mut path: String) -> PathBuf {
        use path_absolutize::*;

        if path.starts_with("~/") {
            let user = std::env::var("USER")
                .expect("Failed to get user name!");
            let home_path = if &user == "root" {
                String::from("/root/")
            } else {
                format!("/home/{}/", user)
            };

            path.remove(0);
            path.remove(0);
            path.insert_str(0, &home_path);
        }

        PathBuf::from(path)
            .absolutize()
            .expect("Failed to get absolute path!")
            .to_path_buf()
    }
}

impl Default for FileState {
    fn default() -> Self {
        FileState {
            path: PathBuf::default(),
            content: Arc::new(Mutex::new(Vec::new())),
            stylized: Arc::new(Mutex::new(Vec::new())),
            theme: ThemeSet::load_defaults().themes["base16-ocean.dark"].to_owned(),
            syntax_set: SyntaxSet::load_defaults_newlines(),
            background_color: None,
            file_modified: Arc::new(Mutex::new(false))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_test() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let mut file_state = FileState::default();
            file_state.init(String::from("/home/spring/test.el")).await?;

            println!("{:#?}", file_state.content);

            Ok::<(), AppError>(())
        }).unwrap();
    }
}
