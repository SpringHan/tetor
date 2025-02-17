// Frame

use std::sync::Arc;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame
};
use tokio::runtime::Runtime;

use crate::{app::App, error::{AppError, AppResult}};

use super::{info_line::InfoLine, Editor};

pub fn main_frame(frame: &mut Frame, app: &mut App, rt: &Runtime) -> AppResult<()> {
    // TODO: Display range modify.
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints(vec![
            Constraint::Percentage(98),
            Constraint::Percentage(2)
        ])
        .split(frame.size());

    // Update the content used to render
    let to_update = app.editor_state.update(main_layout[0]);
    if app.update_stylized || to_update {
        rt.block_on(async {
            app.file_state.refresh_stylized(
                app.editor_state.offset(),
                main_layout[0].height
            ).await?;

            if app.update_stylized {
                let mut search_ref = app.search_ref().lock().await;
                if search_ref.has_history() {
                    search_ref.clear();
                }
            }

            Ok::<(), AppError>(())
        })?;

        app.update_stylized = false;
    }

    // Render frame
    let editor = Editor::new(
        Arc::clone(app.file_state.stylized_ref()),
        Arc::clone(app.search_ref()),
        app.get_bg()?,
        app.app_errors.empty()
    );

    let info_line = InfoLine::from((&mut *app, &*rt));

    frame.render_stateful_widget(
        editor,
        main_layout[0],
        &mut app.editor_state
    );

    frame.render_widget(info_line, main_layout[1]);

    Ok(())
}
