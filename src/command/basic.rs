// Basic

use std::mem::swap;

use crossterm::event::KeyCode;

use crate::{app::App, error::{AppResult, ErrorType}, ui::ModalType};

use super::{command_type::CursorMoveType, CommandPrior};

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

pub async fn delete_char(app: &mut App) -> AppResult<()> {
    let cursor = app.editor_state.cursor();
    let mut current_line = app.file_state
        .get_lines(cursor.1, cursor.1)
        .await?;

    // Delete current line
    if current_line.len() == 1 && current_line[0] == "\n" {
        let file_length = (app.file_state.content_ref().lock().await.len() - 1) as u16;
        app.file_state.modify_lines(cursor.1, cursor.1, Vec::new()).await?;

        if cursor.1 >= file_length {
            app.editor_state.cursor_mut().1 = file_length - 1;
        }

        return Ok(())
    }

    current_line[0].remove(cursor.0 as usize);

    app.file_state.modify_lines(cursor.1, cursor.1, current_line).await?;

    Ok(())
}

pub async fn delete(app: &mut App, key: Option<KeyCode>) -> AppResult<()> {
    let state = &mut app.editor_state;
    let cursor = state.cursor();

    if key.is_none() {
        match state.mark() {
            Some(mark_pos) => {
                if mark_pos == cursor {
                    *state.mark_mut() = None;
                    return Ok(())
                }

                // Delete marked region
                let (mut start, mut end) = (mark_pos, cursor);
                compare_swap(&mut start, &mut end);

                let origin_lines = app.file_state.get_lines(start.1, end.1).await?;
                let mut new_line = String::new();

                new_line.push_str(&origin_lines[0][0..(start.0 as usize)]);
                new_line.push_str(
                    &origin_lines
                        .last()
                        .expect("Error code 1 at delete in basic.rs!")
                        [(end.0 as usize)..]
                );

                app.file_state.modify_lines(start.1, end.1, vec![new_line]).await?;
                *state.cursor_mut() = start;
            },
            None => {
                app.prior_command = CommandPrior::Delete;
            },
        }

        return Ok(())
	  }

    // TODO: Make the key customizable
    match key.unwrap() {
        KeyCode::Char('d') => {
            app.file_state.modify_lines(cursor.1, cursor.1, Vec::new()).await?;

            let file_length = app.file_state.content_ref().lock().await.len();

            if cursor.1 > file_length as u16 {
                state.cursor_mut().1 -= 1;
            }
        },
        KeyCode::Tab => {
            app.file_state.modify_lines(
                cursor.1,
                cursor.1,
                vec![String::from("\n")]
            ).await?;
        },
        _ => return Err(
            ErrorType::Specific(
                String::from("Invalid key command")
            ).pack()
        )
    }

    state.cursor_mut().0 = 0;

    Ok(())
}

#[inline]
fn compare_swap<T>(small: &mut (T, T), big: &mut (T, T))
where T: PartialEq + PartialOrd + Copy
{
    if small.1 > big.1 {
        swap(small, big);
    }

    if small.1 == big.1 && small.0 > big.0 {
        swap(small, big);
    }
}

pub async fn change(app: &mut App, key: Option<KeyCode>) -> AppResult<()> {
    if key.is_none() {
        delete(app, None).await?;

        if app.prior_command == CommandPrior::Delete {
            app.prior_command = CommandPrior::Change;
        } else {
            app.editor_state.modal.switch_insert();
        }

        return Ok(())
    }

    match key.unwrap() {
        KeyCode::Char('c') => delete(app, Some(KeyCode::Char('d'))).await?,
        KeyCode::Tab => delete(app, Some(KeyCode::Tab)).await?,
        _ => return Err(
            ErrorType::Specific(
                String::from("Invalid key command")
            ).pack()
        )
    }

    app.editor_state.modal.switch_insert();

    Ok(())
}
