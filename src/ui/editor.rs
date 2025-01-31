// Editord

use ratatui::{
    text::Line,
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget}
};
use tokio::sync::Mutex;

use std::{mem::swap, sync::Arc};

use crate::fs::{ContentLine, LineVec};

use super::modal::Modal;

#[derive(Debug, Default, Clone, Copy)]
pub struct EditorState {
    cursor_pos: (usize, usize),
    scroll_offset: usize,
    modal: Modal
}

#[derive(Debug)]
pub struct Editor {
    lines: Arc<Mutex<LineVec>>,
}

impl Editor {
    pub fn new(content: LineVec) -> Self {
        Editor {
            lines: Arc::new(Mutex::new(content)),
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
                acc.extend(line.get_iter()
                    .map(|(style, content)| Line::styled(content.to_owned(), *style))
                );
                acc
            });

        Paragraph::new(text)
            .block(block)
            .render(area, buf);

        // Render cursor
        let (row, col) = state.cursor_pos;
        let x = area.x + col as u16 + 1;
        let y = area.y + (row - state.scroll_offset) as u16 + 1;
        let point = buf.get_mut(x, y);
        swap(&mut point.fg, &mut point.bg);
    }
}
