// Basic

use std::mem::swap;

use crossterm::event::KeyCode;

use crate::{app::App, error::{AppResult, ErrorType}, fs::ContentLine, ui::ModalType};

use super::{command_type::CursorMoveType, CommandPrior};

pub async fn change_modal(
    app: &mut App,
    modal: ModalType,
    cursor_move: CursorMoveType
) -> AppResult<()>
{
    // TODO: add check for mark point
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

pub async fn replace_char(app: &mut App, key: Option<KeyCode>) -> AppResult<()> {
    if key.is_none() {
        app.prior_command = CommandPrior::ReplaceChar;

        return Ok(())
    }

    app.prior_command = CommandPrior::None;

    if let KeyCode::Char(_key) = key.unwrap() {
        let cursor = app.editor_state.cursor();
        let mut line = app.file_state
            .get_lines(cursor.1, cursor.1)
            .await?[0]
            .to_owned()
            .into_bytes();

        line[cursor.0 as usize] = _key as u8;

        app.file_state.modify_lines(
            cursor.1,
            cursor.1,
            vec![
                String::from_utf8(line)
                    .expect("Error code 2 at replace_char in basic.rs.")
            ]
        ).await?;
    }

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
                // The true end position of the marked region equals to cursor_x - 1
                if end == mark_pos {
                    end.0 += 1;
                }

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

    // NOTE: Avoid the occurred error makes this value cannot be reset.
    app.prior_command = CommandPrior::None;

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

    app.prior_command = CommandPrior::None;

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

pub fn mark(app: &mut App, key: Option<KeyCode>) -> AppResult<()> {
    if key.is_none() {
        app.prior_command = CommandPrior::Mark;

        return Ok(())
    }

    app.prior_command = CommandPrior::None;
    let state = &mut app.editor_state;

    match key.unwrap() {
        KeyCode::Char('m') => *state.mark_mut() = Some(state.cursor()),
        // KeyCode::Char('l') => {
        //     let cursor = state.cursor();
        //     let line_length = app.file_state.get_lines(cursor.1, cursor.1).await?[0].len();

        //     *state.mark_mut() = Some((0, cursor.1));
        //     state.cursor_mut().1 = line_length as u16 - 1;
        // },
        _ => return Err(
            ErrorType::Specific(
                String::from("Invalid key command")
            ).pack()
        )
    }

    Ok(())
}

pub fn cancel_mark(app: &mut App) -> AppResult<()> {
    if app.editor_state.mark().is_some() {
        *app.editor_state.mark_mut() = None;
    }
    
    Ok(())
}

pub async fn newline(app: &mut App, down: bool) -> AppResult<()> {
    let mut file_content = app.file_state.content_ref().lock().await;
    let cursor = app.editor_state.cursor();
    let mut line_after = cursor.1 as usize;

    let new_line = ContentLine::new(vec![
        (ratatui::style::Style::default(), String::from("\n"))
    ]);

    if down {
        line_after += 1;
    }

    if line_after >= file_content.len() {
        file_content.push(new_line);
        drop(file_content);

        *app.editor_state.cursor_mut() = (0, line_after as u16);
        app.get_modal().switch_insert();
        return Ok(())
    }

    file_content.insert(line_after, new_line);
    drop(file_content);

    app.file_state.file_modify().await;
    app.get_modal().switch_insert();

    Ok(())
}

pub async fn save(app: &mut App) -> AppResult<()> {
    app.file_state.save_content().await?;

    Ok(())
}

pub async fn quit(app: &mut App, key: Option<KeyCode>) -> AppResult<()> {
    if key.is_none() {
        if app.file_state.not_save().await {
            app.prior_command = CommandPrior::Quit(false);
        } else {
            app.prior_command = CommandPrior::Quit(true);
        }

        return Ok(())
    }

    app.prior_command = CommandPrior::None;

    if let KeyCode::Char(_key) = key.unwrap() {
        if _key == 'y' {
            app.prior_command = CommandPrior::Quit(true);
        }
    }

    Ok(())
}
