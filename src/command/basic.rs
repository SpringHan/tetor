// Basic

use crate::{app::App, error::{AppError, AppResult}, ui::ModalType};

use super::command_type::{CursorMoveType, MotionDirection};

pub(super) async fn change_modal(
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

    if cursor_move == CursorMoveType::Num(0) {
        return Ok(())
    }

    let cursor_pos = app.editor_state.cursor();
    let cursor_x = app.editor_state.cursor_x();
    let line_length = app.file_state.get_lines(cursor_pos.1, cursor_pos.1)
        .await?[0]
        .len();

    match cursor_move {
        CursorMoveType::Num(i) => {
            if cursor_pos.0 + i >= line_length as u16 {
                *cursor_x = line_length as u16;

                return Ok(())
            }

            *cursor_x += i;
        },
        CursorMoveType::Beg => *cursor_x += 0,
        CursorMoveType::End => *cursor_x = line_length as u16,
    }

    Ok(())
}

pub(super) async fn move_cursor(
    app: &mut App,
    direction: MotionDirection,
    cursor_move: CursorMoveType
) -> AppResult<()>
{
    Ok(())
}

pub(super) async fn goto(
    app: &mut App,
    move_in_buffer: bool,
    cursor_move: CursorMoveType
) -> AppResult<()>
{
    Ok(())
}

pub(super) async fn insert_char(app: &mut App, key: char) -> AppResult<()> {
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

    Ok(())
}
