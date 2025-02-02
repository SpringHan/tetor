// Frame

use crate::app::App;

use ratatui::{layout::{Constraint, Direction, Layout}, Frame};

use super::Editor;

pub fn main_frame(frame: &mut Frame, app: &mut App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage(98),
            Constraint::Percentage(2)
        ])
        .split(frame.size());

    let file_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(5),
            Constraint::Percentage(95)
        ])
        .split(main_layout[0]);

    let editor = Editor::new(app.get_content(), app.get_bg());

    frame.render_stateful_widget(
        editor,
        file_layout[1],
        &mut app.editor_state
    );
}
