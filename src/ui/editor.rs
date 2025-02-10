// Editord

use ratatui::{
    style::Color, text::{Line, Span}, widgets::{Block, Borders, Paragraph, StatefulWidget, Widget}
};
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
    background_color: Color
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
    pub fn new(content: Arc<Mutex<LineVec>>, bg: Color) -> Self {
        Editor {
            lines: content,
            background_color: bg
        }
    }
}

impl StatefulWidget for Editor {
    type State = EditorState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State
    )
    {
        let block = Block::default()
            .borders(Borders::LEFT);

        if state.editor_height.is_none() {
            state.editor_height = Some(area.height as isize);
        }

        // TODO: Add mark region display

        // Adjust scroll_offset & cursor position.
        if state.cursor_pos.1 < state.scroll_offset as u16 {
            if state.scrolling {
                state.cursor_pos.1 = state.scroll_offset as u16;

                state.scrolling = false;
            } else {
                state.scroll_offset = state.cursor_pos.1 as usize;
            }
        } else if state.cursor_pos.1 > area.height + state.scroll_offset as u16 {
            if state.scrolling {
                state.cursor_pos.1 = state.scroll_offset as u16 + area.height - 1;

                state.scrolling = false;
            } else {
                state.scroll_offset = (state.cursor_pos.1 - area.height / 2) as usize;
            }
        }

        // state.scroll_offset = 0;
        // state.cursor_pos = (0, 0);

        let text = self.lines.blocking_lock()[state.scroll_offset..]
            .iter()
            .take(area.height as usize - 2)
            .fold(Vec::new(), |mut acc, line| {
                acc.push(Line::from(
                    line.get_iter()
                        .map(|(style, content)| Span::styled(content.to_owned(), *style))
                        .collect::<Vec<_>>()
                ));
                acc
            });

        Paragraph::new(text)
            .block(block)
            .render(area, buf);

        // Render cursor
        let (x, y) = state.cursor_pos;

        // BUG: Bug here
        let point = buf.get_mut(x + area.x, y - state.scroll_offset as u16);

        point.bg = Color::White;
        point.fg = self.background_color;
    }
}
