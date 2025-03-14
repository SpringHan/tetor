// Basic

use ratatui::crossterm::event::KeyCode;

use crate::{
    app::App,
    error::{AppResult, ErrorType},
    ui::CommandEdit,
    utils::cursor_compare_swap
};

use super::{command_type::CursorMoveType, CommandPrior};

pub async fn change_insert(
    app: &mut App,
    cursor_move: CursorMoveType
) -> AppResult<bool>
{
    let mark = app.editor_state.mark();
    loop {
        if mark.is_some() {
            let mut start = mark.unwrap();
            let mut end = app.editor_state.cursor();
            cursor_compare_swap(&mut start, &mut end);

            let cursor_after = match cursor_move {
                CursorMoveType::Beg => start,
                CursorMoveType::End => end,
                CursorMoveType::Num(_) => {
                    *app.editor_state.mark_mut() = None;
                    break;
                },
            };

            *app.editor_state.cursor_mut() = cursor_after;
            *app.editor_state.mark_mut() = None;
            app.get_modal().switch_insert();
            return Ok(false)
        }

        break;
    }

    move_cursor(app, true, cursor_move).await?;

    app.get_modal().switch_insert();

    Ok(false)
}

pub async fn move_cursor(
    app: &mut App,
    within_line: bool,
    cursor_move: CursorMoveType
) -> AppResult<bool>
{
    *app.editor_state.cursor_mut() = cursor_move.after_move(
        within_line,
        app.editor_state.cursor(),
        &mut app.file_state
    ).await?;

    Ok(false)
}

pub async fn page_scroll(app: &mut App, scroll: isize) -> bool {
    let editor_state = &mut app.editor_state;
    let scroll_after = (editor_state.offset() as isize) + (scroll * editor_state.height());

    if scroll_after < 0 {
        *editor_state.offset_mut() = 0;
        editor_state.scrolling = true;

        return true
    }


    let scroll_after = scroll_after as usize;

    let file_length = app.file_state.content_ref().lock().await.len() as isize;
    let max_offset = file_length - editor_state.height();

    if max_offset < 0 {
        return false
    }

    *editor_state.offset_mut() = if scroll_after >= max_offset as usize {
        max_offset as usize
    } else {
        scroll_after
    };

    app.editor_state.scrolling = true;

    true
}

pub async fn insert_char(app: &mut App, key: char) -> AppResult<bool> {
    let mut use_space_tab = false;
    let cursor_pos = app.editor_state.cursor();
    let mut edit_line = app.file_state.get_lines(
        cursor_pos.1,
        cursor_pos.1
    ).await?;

    // Handle tab insert
    if key == '\t' && !app.options().tab_indent {
        use_space_tab = true;
        edit_line[0].insert_str(cursor_pos.0 as usize, "    ");
    } else if key == '\n' {
        let temp_line = edit_line[0][cursor_pos.0 as usize..].to_owned();
        edit_line[0].replace_range((cursor_pos.0 as usize).., "\n");
        edit_line.push(temp_line);
    } else {
        edit_line[0].insert(cursor_pos.0 as usize, key);
    }

    app.file_state.modify_lines(
        cursor_pos.1,
        cursor_pos.1,
        edit_line
    ).await?;

    if use_space_tab {
        app.editor_state.cursor_mut().0 += 4;
        return Ok(true)
    }

    if key == '\n' {
        let cursor = app.editor_state.cursor_mut();
        cursor.0 = 0;
        cursor.1 += 1;
        return Ok(true)
    }

    app.editor_state.cursor_mut().0 += 1;

    Ok(true)
}

pub async fn delete_char(app: &mut App) -> AppResult<bool> {
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

        app.file_state.file_modify().await;
        return Ok(true)
    }

    current_line[0].remove(cursor.0 as usize);

    app.file_state.modify_lines(cursor.1, cursor.1, current_line).await?;
    app.file_state.file_modify().await;

    Ok(true)
}

pub async fn replace_char(app: &mut App, key: Option<KeyCode>) -> AppResult<bool> {
    if key.is_none() {
        app.prior_command = CommandPrior::ReplaceChar;

        return Ok(false)
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

    Ok(true)
}

pub async fn delete(app: &mut App, key: Option<KeyCode>) -> AppResult<bool> {
    let state = &mut app.editor_state;
    let cursor = state.cursor();

    if key.is_none() {
        match state.mark() {
            Some(mark_pos) => {
                if mark_pos == cursor {
                    *state.mark_mut() = None;
                    return Ok(false)
                }

                // Delete marked region
                let (mut start, mut end) = (mark_pos, cursor);
                cursor_compare_swap(&mut start, &mut end);

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
                *state.mark_mut() = None;

                return Ok(true)
            },
            None => {
                app.prior_command = CommandPrior::Delete;
                return Ok(false)
            },
        }
	  }

    // NOTE: Avoid the occurred error makes this value cannot be reset.
    app.prior_command = CommandPrior::None;

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

    Ok(true)
}

pub async fn change(app: &mut App, key: Option<KeyCode>) -> AppResult<bool> {
    if key.is_none() {
        let to_update = delete(app, None).await?;

        if app.prior_command == CommandPrior::Delete {
            app.prior_command = CommandPrior::Change;
        } else {
            app.editor_state.modal.switch_insert();
        }

        return Ok(to_update)
    }

    app.prior_command = CommandPrior::None;

    let to_update = match key.unwrap() {
        KeyCode::Char('c') => delete(app, Some(KeyCode::Char('d'))).await?,
        KeyCode::Tab => delete(app, Some(KeyCode::Tab)).await?,
        _ => return Err(
            ErrorType::Specific(
                String::from("Invalid key command")
            ).pack()
        )
    };

    app.editor_state.modal.switch_insert();

    Ok(to_update)
}

pub fn mark(app: &mut App, key: Option<KeyCode>) -> AppResult<bool> {
    if key.is_none() {
        app.prior_command = CommandPrior::Mark;

        return Ok(false)
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

    Ok(false)
}

pub async fn newline(app: &mut App, down: bool) -> bool {
    let mut file_content = app.file_state.content_ref().lock().await;
    let cursor = app.editor_state.cursor();
    let mut line_after = cursor.1 as usize;

    let new_line = String::from("\n");

    // NOTE: When the file is empty, newline will only create a line.
    if file_content.is_empty() {
        file_content.push(new_line);
        return true
    }

    if down {
        line_after += 1;
    }

    app.file_state.file_modify().await;
    *app.editor_state.cursor_mut() = (0, line_after as u16);

    if line_after >= file_content.len() {
        file_content.push(new_line);
        drop(file_content);

        app.get_modal().switch_insert();
        return true
    }

    file_content.insert(line_after, new_line);
    drop(file_content);

    app.get_modal().switch_insert();
    true
}

pub async fn backward_char(app: &mut App) -> AppResult<bool> {
    let cursor = app.editor_state.cursor();

    if cursor.0 == 0 {
        if cursor.1 == 0 {
            return Ok(false)
        }

        let mut lines = app.file_state.get_lines(cursor.1 - 1, cursor.1).await?;
        *app.editor_state.cursor_mut() = (
            lines[0].len() as u16 - 1,
            cursor.1 - 1
        );

        lines[0].pop();
        app.file_state.modify_lines(
            cursor.1 - 1,
            cursor.1,
            vec![lines.join("")]
        ).await?;

        app.file_state.file_modify().await;
        return Ok(true)
    }

    let mut modified_line = app.file_state.get_lines(cursor.1, cursor.1).await?;
    
    modified_line[0].remove(cursor.0 as usize - 1);

    app.file_state.modify_lines(cursor.1, cursor.1, modified_line).await?;
    app.file_state.file_modify().await;
    app.editor_state.cursor_mut().0 -= 1;

    Ok(true)
}

pub async fn search(app: &mut App, pattern: Option<String>) -> AppResult<bool> {
    if pattern.is_none() {
        app.command_edit = CommandEdit::new(
            String::from("/"),
            CommandPrior::Search(String::new())
        );

        return Ok(false)
    }

    let mut pat = pattern.unwrap();
    if pat.starts_with("/") {
        pat.remove(0);
    }

    let content = app.file_state.content_ref().lock().await;
    let mut indicates: Vec<(u16, u16)> = Vec::new();

    let mut line_nr = 0;
    for line in content.iter() {
        for indicate in line.match_indices(&pat) {
            indicates.push((indicate.0 as u16, line_nr));
        }

        line_nr += 1;
    }

    if indicates.is_empty() {
        app.prior_command = CommandPrior::None;
        return Ok(false)
    }


    // Select the next nearest item
    let cursor = app.editor_state.cursor();
    let mut search_result = app.search_ref().lock().await;
    search_result.set(pat, indicates.into_iter());

    let cursor_after = search_result.nearest_next(cursor).unwrap();
    drop(search_result);
    *app.editor_state.cursor_mut() = cursor_after;

    app.prior_command = CommandPrior::None;

    Ok(false)
}

pub async fn search_jump(app: &mut App, next: bool) -> AppResult<bool> {
    let mut search_ref = app.search_ref().lock().await;
    let indicates = search_ref.indicates();

    if indicates.is_empty() {
        return Ok(false)
    }

    // Update current select index
    let move_way = if next { 1 } else { -1 };

    *search_ref.selected_mut() = match search_ref.selected() {
        None => {
            if next {
                Some(0)
            } else {
                Some(indicates.len() - 1)
            }
        },
        Some(i) => {
            let idx_after = i as isize + move_way;
            if idx_after < 0 {
                Some(indicates.len() - 1)
            } else if idx_after as usize == indicates.len() {
                Some(0)
            } else {
                Some(idx_after as usize)
            }
        }
    };

    // Move cursor
    let cursor = search_ref.current_indicate().unwrap();
    drop(search_ref);

    *app.editor_state.cursor_mut() = cursor;

    Ok(false)
}

/// The general command binded for ESC key.
pub async fn escape_command(app: &mut App) -> AppResult<bool> {
    // Cancel mark
    if app.editor_state.mark().is_some() {
        *app.editor_state.mark_mut() = None;

        return Ok(false)
    }

    // Clear search results
    let mut search_ref = app.search_ref().lock().await;
    if search_ref.has_history() {
        search_ref.clear();
    }

    Ok(false)
}

pub async fn save(app: &mut App) -> AppResult<bool> {
    app.file_state.save_content().await?;

    Ok(false)
}

pub async fn quit(app: &mut App, key: Option<KeyCode>) -> bool {
    if key.is_none() {
        if app.file_state.not_save().await {
            app.prior_command = CommandPrior::Quit(false);
            app.ask_msg = Some(String::from("File has not been saved, still quit?"));
        } else {
            app.prior_command = CommandPrior::Quit(true);
        }

        return false
    }

    app.prior_command = CommandPrior::None;
    app.ask_msg = None;

    if let KeyCode::Char(_key) = key.unwrap() {
        if _key == 'y' {
            app.prior_command = CommandPrior::Quit(true);
        }
    }

    false
}
