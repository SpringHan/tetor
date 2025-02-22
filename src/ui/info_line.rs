// Info Line

use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Widget
};

use tokio::runtime::Runtime;

use crate::{app::App, command::CommandPrior, ui::ModalType};

use super::CommandEdit;

#[derive(Debug, Clone)]
pub struct InfoLine<'a> {
    msg: Line<'a>,
    modified_sign: Line<'a>
}

impl<'a> InfoLine<'a> {
    fn make_yes_span() -> Span<'a> {
        Span::styled(
            "y",
            Style::new()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        )
    }
}

impl<'a> From<(&mut App, &Runtime)> for InfoLine<'a> {
    fn from(value: (&mut App, &Runtime)) -> Self {
        let rt = value.1;
        let app = value.0;
        let mut msg: Vec<Span> = Vec::new();
        let mut sign: Vec<Span> = Vec::new();

        loop {
            // Error msg
            if !app.app_errors.empty() {
                app.prior_command = CommandPrior::ConfirmError;
                msg.push(Span::styled(
                    app.app_errors.get_first() ,
                    Style::new().fg(Color::Red)
                ));
                msg.push(Span::styled(" ", Style::new().bg(Color::White)));

                break;
            }

            // Ask msg
            if let Some(ref _msg) = app.ask_msg {
                msg.push(Span::from(_msg.to_owned()));
                msg.push(Span::from(" ("));
                msg.push(Self::make_yes_span());
                msg.push(Span::from(" for yes)"));

                break;
            }

            // Command Line Editing
            if let CommandEdit::Some(ref content, ref cursor, _) = app.command_edit {
                if *cursor == content.len() {
                    msg.push(Span::from(content.to_owned()));
                    msg.push(Span::styled(" ", Style::new().bg(Color::White)));

                    break;
                }

                let mut i = 0;
                for _char in content.chars() {
                    msg.push(Span::styled(
                        String::from(_char),
                        if i == *cursor {
                            Style::new().bg(Color::White)
                                .fg(Color::Black)
                        } else {
                            Style::default()
                        }
                    ));

                    i += 1;
                }

                break;
            }

            // Modal state display
            if app.editor_state.modal.modal() == ModalType::Insert {
                msg.push(Span::styled(
                    String::from(" --INSERT--"),
                    Style::new().add_modifier(Modifier::BOLD)
                ));
            }

            let search_ref = app.search_ref().blocking_lock();
            if search_ref.has_history() && search_ref.selected() != None {
                sign.push(Span::from(format!(
                    "[{}/{}] ",
                    search_ref.selected().unwrap() + 1,
                    search_ref.indicates().len()
                )));
            }

            // File modification state
            sign.push(Span::styled(
                if rt.block_on(app.file_state.not_save()) {
                    "*  "    
                } else {
                    "   "
                },
                Style::new().add_modifier(Modifier::BOLD)
            ));

            break;
        }

        Self {
            msg: Line::from(msg).alignment(Alignment::Left),
            modified_sign: Line::from(sign).alignment(Alignment::Right)
        }
    }
}


impl<'a> Widget for InfoLine<'a> {
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer
    )
    {
        self.msg.render(area, buf);
        self.modified_sign.render(area, buf);
    }
}
