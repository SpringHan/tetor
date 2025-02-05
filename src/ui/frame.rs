// Frame

use crate::app::App;

use ratatui::{layout::{Constraint, Direction, Layout}, Frame};

use super::Editor;

pub fn main_frame(frame: &mut Frame, app: &mut App) {
    // TODO: Display range modify.
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints(vec![
            Constraint::Percentage(99),
            Constraint::Percentage(1)
        ])
        .split(frame.size());

    let file_layout = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints(vec![
            Constraint::Percentage(5),
            Constraint::Percentage(95)
        ])
        .split(main_layout[0]);

    let editor = Editor::new(
        std::sync::Arc::clone(app.file_state.content_ref()),
        app.get_bg()
    );

    frame.render_stateful_widget(
        editor,
        file_layout[1],
        &mut app.editor_state
    );
}
