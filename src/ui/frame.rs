// Frame

use crate::app::App;

use ratatui::{layout::{Constraint, Direction, Layout}, Frame};
use tokio::runtime::Runtime;

use super::{info_line::InfoLine, Editor};

pub fn main_frame(frame: &mut Frame, app: &mut App, rt: &Runtime) {
    // TODO: Display range modify.
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints(vec![
            Constraint::Percentage(98),
            Constraint::Percentage(2)
        ])
        .split(frame.size());

    let editor = Editor::new(
        std::sync::Arc::clone(app.file_state.content_ref()),
        app.get_bg(),
        app.app_errors.empty()
    );

    let info_line = InfoLine::from((&mut *app, &*rt));

    frame.render_stateful_widget(
        editor,
        main_layout[0],
        &mut app.editor_state
    );

    frame.render_widget(info_line, main_layout[1]);
}
