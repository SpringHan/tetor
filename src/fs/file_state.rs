// File State

use super::type_convert::{ColorConvert, StyleConvert};
use crate::error::{self, AppError, AppResult, ErrorType};

use ratatui::style::{Color, Style};
use tokio::{fs, sync::Mutex};
use tokio::sync::mpsc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
#[derive(Debug, Clone)]
pub struct ContentLine(StylizedContent);

// TODO: Do not load all the file when the file is too large
#[derive(Debug)]
pub struct FileState {
    pub background_color: Color,
    content: Arc<Mutex<LineVec>>,
    file_modified: Arc<Mutex<bool>>,

    path: PathBuf,
    theme: Theme,
    syntax_set: SyntaxSet
}

impl ContentLine {
    pub fn new(line: StylizedContent) -> Self {
        ContentLine(line)
    }

    pub fn get_iter<'a>(&'a self) -> impl Iterator<Item = &'a (Style, String)> {
        self.0.iter()
    }
}

impl Into<String> for ContentLine {
    fn into(self) -> String {
        self.0.into_iter()
            .map(|(_, span)| span)
            .collect::<Vec<_>>()
            .join("")
    }
}

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

    pub async fn init<P: AsRef<Path>>(&mut self, path: P) -> AppResult<()> {
        let mut file = fs::File::open(path.as_ref().to_owned()).await?;
        let (tx, rx) = mpsc::unbounded_channel();

        let read_task = async {
            let mut _content = String::new();

            // TODO: Modify to read line by line
            file.read_to_string(&mut _content).await?;

            if tx.is_closed() {
                return Err(
                    ErrorType::Specific(
                        String::from("Channel closed when initializing file information")
                    ).pack()
                )
            }

            tx.send(_content).unwrap();

            Ok::<(), AppError>(())
        };

        let (read_result, parse_result) = tokio::join!(
            read_task,
            self.parse_content(path.as_ref().to_owned(), rx)
        );

        read_result?;
        self.path = path.as_ref().to_path_buf();
        (*self.content.lock().await, self.background_color) = parse_result?;

        Ok(())
    }

    async fn parse_content<'a, P>(
        &self,
        path: P,
        mut rx: mpsc::UnboundedReceiver<String>
    ) -> AppResult<(LineVec, Color)>
    where P: AsRef<Path>
    {
        let find_syntax = self.syntax_set.find_syntax_for_file(
            path.as_ref()
        )?;

        let mut result: LineVec = Vec::new();
        let mut get_bg = false;
        let mut background_color: Color = Color::default();

        let mut h = if find_syntax.is_some() {
            Some(HighlightLines::new(find_syntax.unwrap(), &self.theme))
        } else {
            None
        };

        if let Some(content) = rx.recv().await {
            for line in LinesWithEndings::from(&content) {
                // Highligth line
                if let Some(ref mut _h) = h {
                    let ranges = _h.highlight_line(line, &self.syntax_set)
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
                    vec![(Style::default(), line.to_owned())]
                ));
            }
        }

        Ok((result, background_color))
    }

    /// Get lines from file content with range.
    pub async fn get_lines(&self, from: u16, to: u16) -> AppResult<Vec<String>> {
        let (from, to) = (from as usize, to as usize);
        let file_lines = self.content.lock().await;

        if from > to || to >= file_lines.len() {
            return Err(
                ErrorType::Specific(
                    String::from("Attempt to get lines with wrong range.")
                ).pack()
            )
        }

        Ok(
            file_lines[from..=to].to_owned()
                .into_iter()
                .map(|line| line.into())
                .collect::<Vec<String>>()
        )
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

        if lines.is_empty() {
            for i in from..=to {
                file_lines.remove(i);
            }

            return Ok(())
        }


        // Get highlighted lines
        let (tx, rx) = mpsc::unbounded_channel();

        tx.send(lines.join(""))
            .expect("Error code 2 at modify_lines in file_state.rs!");

        let mut highlighted_lines = self.parse_content(
            self.path.to_owned(),
            rx
        ).await?.0;

        // Simply replace the original lines
        for i in from..=to {
            if i >= file_lines.len() {
                // file_lines.push(highlighted_lines[i - from].to_owned());
                file_lines.push(Self::pop_first(&mut highlighted_lines));
                continue;
            }

            if i - from >= highlighted_lines.len() {
                file_lines.remove(i);
                continue;
            }

            // file_lines[i] = highlighted_lines[i - from].to_owned();
            file_lines[i] = Self::pop_first(&mut highlighted_lines);
        }

        if !highlighted_lines.is_empty() {
            for line in highlighted_lines.into_iter() {
                if to == file_lines.len() - 1 {
                    file_lines.push(line);
                    continue;
                }

                file_lines.insert(to + 1, line);
            }
        }

        self.file_modify().await;

        Ok(())
    }

    pub async fn save_content(&mut self) -> AppResult<()> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .append(true)
            .open(self.path.to_owned()).await?;

        let (tx, mut rx) = mpsc::unbounded_channel::<String>();

        let destyle_task = async {
            for line in self.content.lock().await.iter() {
                tx.send(line.to_owned().into())
                    .expect("Error code 3 at save_content in file_state.rs!");
            }
        };

        let save_task = async {
            while let Some(line) = rx.recv().await {
                file.write(line.as_bytes()).await?;
            }

            Ok::<(), tokio::io::Error>(())
        };

        let (_, save_result) = tokio::join!(
            destyle_task,
            save_task
        );

        save_result?;

        *self.file_modified.lock().await = false;

        Ok(())
    }

    fn pop_first(_vec: &mut Vec<ContentLine>) -> ContentLine {
        let element = _vec[0].to_owned();
        _vec.remove(0);
        element
    }
}

impl Default for FileState {
    fn default() -> Self {
        FileState {
            path: PathBuf::default(),
            content: Arc::new(Mutex::new(Vec::new())),
            theme: ThemeSet::load_defaults().themes["base16-ocean.dark"].to_owned(),
            syntax_set: SyntaxSet::load_defaults_newlines(),
            background_color: Color::default(),
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
            file_state.init(PathBuf::from("/home/spring/test.el")).await?;

            println!("{:#?}", file_state.content);
            // file_state.reset().await;

            Ok::<(), AppError>(())
        }).unwrap();
    }
}
