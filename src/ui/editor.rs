// Editord

use ratatui::{style::{Color, Style}, widgets::StatefulWidget};
use tokio::sync::Mutex;

use std::sync::Arc;

use crate::fs::LineVec;

use super::modal::Modal;

/// The editor state for Editor widget.
/// What needs to be stress on is that cursor_pos is the absolute position for the file.
/// Not the buffer.
#[derive(Debug, Clone, Copy)]
pub struct EditorState {
    cursor_pos: (u16, u16),
    mark_point: Option<(u16, u16)>,

    scroll_offset: usize,

    editor_height: Option<isize>,

    pub modal: Modal,
    pub scrolling: bool
}

#[derive(Debug, Default)]
pub struct Editor {
    lines: Arc<Mutex<LineVec>>,
    background_color: Color,
    render_cursor: bool
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            cursor_pos: (0, 0),
            mark_point: None,

            scrolling: false,
            scroll_offset: 0,

            editor_height: None,
            modal: Modal::default(),
        }
    }
}

impl EditorState {
    pub fn offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn offset_mut(&mut self) -> &mut usize {
        &mut self.scroll_offset
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
}

impl Editor {
    pub fn new(content: Arc<Mutex<LineVec>>, bg: Color, render_cursor: bool) -> Self {
        Editor {
            lines: content,
            background_color: bg,
            render_cursor
        }
    }

    /// Get length of line number in this crate.
    fn nr_length<N: ToString>(nr: N) -> u8 {
        nr.to_string().chars().count() as u8
    }

    /// Core part for render lines.
    fn render_core(
        &mut self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut EditorState
    )
    {
        let text = self.lines.blocking_lock();

        // Update linenr_width
        let linenr_width = {
            let length = Self::nr_length(text.len());
            if length <= 4 {
                4
            } else {
                length
            }
        };

        // Render line number & content
        let mut _y = 0;
        let mut file_line = state.scroll_offset;
        while _y < area.height {
            if let Some(line) = text.get(file_line) {
                let mut current_length = 0;
                let mut current_point = linenr_width as u16 + 1; // Current horizontal position in buf
                let linenr_start = linenr_width - Self::nr_length(file_line + 1);

                // Render line number
                buf.set_string(
                    linenr_start as u16,
                    _y,
                    (file_line + 1).to_string(),
                    Style::default()
                );

                // Render delimiter
                buf.get_mut(current_point, _y).set_symbol("|");
                current_point += 1;

                // TODO: Add display for marked content
                // Render content
                for (style, span) in line.get_iter() {
                    for _char in span.chars() {
                        if current_point == area.width {
                            _y += 1;
                            if _y == area.height {
                                break;
                            }

                            current_point = linenr_width as u16 + 2;
                            buf.get_mut(current_point - 1, _y).set_symbol("|");
                        }

                        let point_buf = buf.get_mut(current_point, _y);
                        if _char != '\n' {
                            point_buf.set_char(_char);
                        }

                        if self.render_cursor &&
                            state.cursor_pos.0 == current_length &&
                            state.cursor_pos.1 == file_line as u16
                        {
                            point_buf.bg = Color::White;
                            point_buf.fg = self.background_color;
                        } else {
                            point_buf.set_style(*style);
                        }

                        current_point += 1;
                        current_length += 1;
                    }
                }

                _y += 1;
                file_line += 1;

                continue;
            }

            break;
        }
    }
}

impl StatefulWidget for Editor {
    type State = EditorState;

    fn render(
        mut self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State
    )
    {
        // Adjust window size
        if state.editor_height.is_none() ||
            state.editor_height.unwrap() != area.height as isize
        {
            state.editor_height = Some(area.height as isize);
        }

        // Adjust scroll_offset & cursor position.
        if state.cursor_pos.1 < state.scroll_offset as u16 {
            if state.scrolling {
                state.cursor_pos.1 = state.scroll_offset as u16;
            } else {
                state.scroll_offset = state.cursor_pos.1 as usize;
            }
        } else if state.cursor_pos.1 >= area.height + state.scroll_offset as u16 {
            if state.scrolling {
                state.cursor_pos.1 = state.scroll_offset as u16 + area.height - 1;
            } else {
                state.scroll_offset = (state.cursor_pos.1 - area.height / 2) as usize;
            }
        }

        // To avoid this variable make impact on other motion
        // after page_scroll reached to edges.
        if state.scrolling {
            state.scrolling = false;
        }

        // Render editor view & cursor
        self.render_core(area, buf, state);
    }
}
