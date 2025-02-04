// Editord

use ratatui::{
    style::Color, text::{Line, Span}, widgets::{Block, Borders, Paragraph, StatefulWidget, Widget}
};
use tokio::sync::Mutex;

use std::sync::Arc;

use crate::fs::LineVec;

use super::modal::Modal;

#[derive(Debug, Default, Clone, Copy)]
pub struct EditorState {
    cursor_pos: (u16, u16),
    scroll_offset: usize,
    pub modal: Modal
}

#[derive(Debug, Default)]
pub struct Editor {
    lines: Arc<Mutex<LineVec>>,
    background_color: Color
}

impl EditorState {
    // TODO: Modify relative function
    pub fn scroll_down(&mut self, line: usize) {
        self.scroll_offset += line;
    }

    pub fn cursor(&self) -> (u16, u16) {
        self.cursor_pos
    }

    pub fn cursor_x(&mut self) -> &mut u16 {
        &mut self.cursor_pos.0
    }

    pub fn cursor_y(&mut self) -> &mut u16 {
        &mut self.cursor_pos.1
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
            .borders(Borders::NONE);

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
        let (_x, _y) = state.cursor_pos;
        // let point = buf.get_mut(1 + area.x, 1);

        let point = buf.get_mut(_x + area.x, _y);

        point.bg = Color::White;
        point.fg = self.background_color;
    }
}
