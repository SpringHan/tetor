// Editord

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::StatefulWidget
};

use tokio::sync::Mutex;

use std::sync::Arc;

use crate::{app::SearchIndicates, utils::cursor_compare_swap, fs::StylizedVec};
use super::modal::Modal;

/// The editor state for Editor widget.
/// What needs to be stress on is that cursor_pos is the absolute position for the file.
/// Not the buffer.
#[derive(Debug, Clone, Copy)]
pub struct EditorState {
    cursor_pos: (u16, u16),
    mark_point: Option<(u16, u16)>,

    vertical_offset: usize,
    horizontal_offset: u16,

    editor_height: Option<isize>,
    file_linenr: usize,

    pub modal: Modal,
    pub scrolling: bool
}

#[derive(Debug)]
pub struct Editor {
    lines: Arc<Mutex<StylizedVec>>,
    search_indicates: Arc<Mutex<SearchIndicates>>,
    background_color: Color,
    render_cursor: bool
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            cursor_pos: (0, 0),
            mark_point: None,

            scrolling: false,
            vertical_offset: 0,
            horizontal_offset: 0,

            file_linenr: 0,
            editor_height: None,
            modal: Modal::default(),
        }
    }
}

impl EditorState {
    pub fn offset(&self) -> usize {
        self.vertical_offset
    }

    pub fn offset_mut(&mut self) -> &mut usize {
        &mut self.vertical_offset
    }

    pub fn cursor(&self) -> (u16, u16) {
        self.cursor_pos
    }

    pub fn cursor_mut(&mut self) -> &mut (u16, u16) {
        &mut self.cursor_pos
    }

    pub fn height(&self) -> isize {
        self.editor_height.expect("Error code 1 at height in editor.rs!")
    }

    pub fn mark(&self) -> Option<(u16, u16)> {
        self.mark_point
    }

    pub fn mark_mut(&mut self) -> &mut Option<(u16, u16)> {
        &mut self.mark_point
    }

    pub fn update_linenr(&mut self, nr: usize) {
        self.file_linenr = nr;
    }

    pub fn update(&mut self, area: Rect) -> bool {
        let mut to_update = false;

        // Adjust window size
        if self.editor_height.is_none() ||
            self.editor_height.unwrap() != area.height as isize
        {
            if !self.editor_height.is_none() {
                to_update = true;
            }
            self.editor_height = Some(area.height as isize);
        }

        // Adjust vertical_offset & cursor position.
        if self.cursor_pos.1 < self.vertical_offset as u16 {
            if self.scrolling {
                self.cursor_pos.1 = self.vertical_offset as u16;
            } else {
                self.vertical_offset = self.cursor_pos.1 as usize;
            }

            to_update = true;
        } else if self.cursor_pos.1 >= area.height + self.vertical_offset as u16 {
            if self.scrolling {
                self.cursor_pos.1 = self.vertical_offset as u16 + area.height - 1;
            } else {
                self.vertical_offset = (self.cursor_pos.1 - area.height / 2) as usize;
            }

            to_update = true;
        }

        // Adjust horizontal_offset
        let noncontent_width = {
            let length = Editor::nr_length(self.file_linenr);
            if length <= 4 {
                4
            } else {
                length
            }
        } as u16 + 2;
        if noncontent_width >= area.width {
            panic!("The editor is not suitable for large files.")
        }

        if self.horizontal_offset > self.cursor_pos.0 {
            self.horizontal_offset = self.cursor_pos.0;
        } else if self.cursor_pos.0 - self.horizontal_offset + 1 >= area.width - noncontent_width {
            self.horizontal_offset = self.cursor_pos.0 - area.width / 2;
        }

        // To avoid this variable make impact on other motion
        // after page_scroll reached to edges.
        if self.scrolling {
            self.scrolling = false;
        }

        to_update
    }
}

impl Editor {
    pub fn new(
        content: Arc<Mutex<StylizedVec>>,
        indicates: Arc<Mutex<SearchIndicates>>,
        bg: Color,
        render_cursor: bool
    ) -> Self {
        Editor {
            lines: content,
            search_indicates: indicates,
            background_color: bg,
            render_cursor
        }
    }

    /// Get length of line number in this crate.
    fn nr_length<N: ToString>(nr: N) -> u8 {
        nr.to_string().chars().count() as u8
    }

    /// Check whether cursor is within the marked region.
    fn within_mark(state: &EditorState, x: u16, y: u16) -> bool {
        if state.mark().is_none() {
            return false
        }

        let mut cursor_start = state.cursor();
        let mut cursor_end = state.mark().unwrap();
        cursor_compare_swap(&mut cursor_start, &mut cursor_end);

        if y < cursor_start.1 || y > cursor_end.1 ||
            x < cursor_start.0 || x >= cursor_end.0
        {
            return false
        }

        true
    }

    fn color_reverse(&self, buf: &mut Buffer, style: Style, x: u16, y: u16) {
        let point_buf = buf.get_mut(x, y);

        point_buf.bg = if let Some(color) = style.fg {
            color
        } else {
            Color::White
        };
        point_buf.fg = self.background_color;
    }
}

impl StatefulWidget for Editor {
    type State = EditorState;

    fn render(
        self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut Self::State
    )
    {
        let text = self.lines.blocking_lock();
        let indicates = self.search_indicates.blocking_lock();

        // Update linenr_width
        let linenr_width = {
            let length = Self::nr_length(state.file_linenr);
            if length <= 4 {
                4
            } else {
                length
            }
        };

        // Render line number & content
        let mut buf_y = 0;
        let mut file_line = state.vertical_offset;
        for line in text.iter() {
            if buf_y >= area.height {
                break;
            }

            let mut current_length = 0; // Same as the value of state.cursor_pos.0
            let mut buf_x = linenr_width as u16 + 2; // Current horizontal position in buffer
            // let mut linenr_string = format!("{}|", file_line + 1);
            let linenr_idx = (linenr_width - Self::nr_length(file_line + 1)) as u16;

            // Render line number & delimiter
            // while linenr_string.len() > 0 {
            //     let _char = linenr_string.remove(0);
            //     if state.horizontal_offset > linenr_idx {
            //         linenr_idx += 1;
            //         continue;
            //     }

            //     buf.get_mut(linenr_idx, buf_y).set_char(_char);
            //     linenr_idx += 1;
            // }
            buf.set_string(
                linenr_idx as u16,
                buf_y,
                (file_line + 1).to_string(),
                Style::default()
            );

            // Render delimiter between line number & content
            buf.get_mut(buf_x, buf_y).set_symbol("|");
            buf_x += 1;

            // Render content
            'line: for (style, span) in line.get_iter() {
                for _char in span.chars() {
                    // Stop rendering current line
                    if buf_x == area.width {
                        break 'line;
                        // buf_y += 1;
                        // if buf_y == area.height {
                        //     return ()
                        // }

                        // buf_x = linenr_width as u16 + 2;
                        // buf.get_mut(buf_x - 1, buf_y).set_symbol("|");
                    }

                    if current_length < state.horizontal_offset {
                        current_length += 1;
                        continue;
                    }

                    let point_buf = buf.get_mut(buf_x, buf_y);
                    if _char != '\n' && _char != '\t' {
                        point_buf.set_char(_char);
                    }

                    loop {
                        // Cursor
                        if self.render_cursor &&
                            state.cursor_pos.0 == current_length &&
                            state.cursor_pos.1 == file_line as u16
                        {
                            point_buf.bg = Color::White;
                            point_buf.fg = self.background_color;
                            break;
                        }

                        // Search indicates
                        if Self::within_mark(state, current_length, file_line as u16) ||
                            indicates.indicates_find((
                                current_length,
                                file_line as u16
                            ))
                        {
                            self.color_reverse(buf, *style, buf_x, buf_y);
                            break;
                        }

                        point_buf.set_style(*style);
                        break;
                    }

                    current_length += 1;

                    // Deal with the display of tabs
                    if _char == '\t' {
                        for _ in 0..4 {
                            buf_x += 1;

                            if buf_x < area.width &&
                                Self::within_mark(state, current_length, file_line as u16)
                            {
                                self.color_reverse(buf, *style, buf_x, buf_y);
                            }

                            // Avoid out of range panic
                            if buf_x == area.width {
                                break 'line;
                                // buf_y += 1;
                                // if buf_y == area.height {
                                //     return ()
                                // }

                                // buf_x = 0;
                            }
                        }

                        continue;
                    }

                    buf_x += 1;
                }
            }
            
            buf_y += 1;
            file_line += 1;
        }
    }
}
