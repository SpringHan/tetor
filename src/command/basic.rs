// Basic

use crate::{app::App, error::AppResult, ui::ModalType};

use super::command_type::CursorMoveType;

pub async fn change_modal(
    app: &mut App,
    modal: ModalType,
    cursor_move: CursorMoveType
) -> AppResult<()>
{
    if modal == ModalType::Normal {
        app.get_modal().switch_normal();
    } else {
        app.get_modal().switch_insert();
    }

    move_cursor(app, true, cursor_move).await?;

    Ok(())
}

pub async fn move_cursor(
    app: &mut App,
    within_line: bool,
    cursor_move: CursorMoveType
) -> AppResult<()>
{
    *app.editor_state.cursor_mut() = cursor_move.after_move(
        within_line,
        app.editor_state.cursor(),
        &mut app.file_state
    ).await?;

    Ok(())
}

pub async fn page_scroll(app: &mut App, scroll: isize) -> AppResult<()> {
    let editor_state = &mut app.editor_state;
    let scroll_after = (editor_state.offset() as isize) + (scroll * editor_state.height());

    if scroll_after < 0 {
        *editor_state.offset_mut() = 0;
        editor_state.scrolling = true;

        return Ok(())
    }


    let scroll_after = scroll_after as usize;

    let file_length = app.file_state.content_ref().lock().await.len() as isize;
    let max_offset = file_length - editor_state.height();

    if max_offset < 0 {
        return Ok(())
    }

    *editor_state.offset_mut() = if scroll_after >= max_offset as usize {
        max_offset as usize
    } else {
        scroll_after
    };

    app.editor_state.scrolling = true;
    
    Ok(())
}

pub async fn insert_char(app: &mut App, key: char) -> AppResult<()> {
    let cursor_pos = app.editor_state.cursor();
    let mut edit_line = app.file_state.get_lines(
        cursor_pos.1,
        cursor_pos.1
    ).await?;

    edit_line[0].insert(cursor_pos.0 as usize, key);

    app.file_state.modify_lines(
        cursor_pos.1,
        cursor_pos.1,
        edit_line
    ).await?;

    app.editor_state.cursor_mut().0 += 1;

    Ok(())
}
