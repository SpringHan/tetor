// Info Line

use ratatui::{layout::Alignment, style::{Color, Modifier, Style}, text::{Line, Span}, widgets::{Paragraph, Widget}};
use tokio::runtime::Runtime;

use crate::{app::App, command::CommandPrior, ui::ModalType};

#[derive(Debug, Clone)]
pub struct InfoLine<'a> {
    msg: Line<'a>,
    modified_sign: Option<Line<'a>>
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
        let mut sign = None;

        loop {
            if !app.app_errors.empty() {
                app.prior_command = CommandPrior::ConfirmError;
                msg.push(Span::styled(
                    app.app_errors.get_first() ,
                    Style::new().fg(Color::Red)
                ));
                break;
            }

            if let Some(ref _msg) = app.ask_msg {
                msg.push(Span::from(_msg.to_owned()));
                msg.push(Span::from(" ("));
                msg.push(Self::make_yes_span());
                msg.push(Span::from(" for yes)"));

                break;
            }

            // TODO: Add search input before modal

            if app.editor_state.modal.modal() == ModalType::Insert {
                msg.push(Span::styled(
                    String::from(" --INSERT--"),
                    Style::new().add_modifier(Modifier::BOLD)
                ));
            }

            if rt.block_on(app.file_state.not_save()) {
                sign = Some(
                    Line::styled(
                        "*  ",
                        Style::new().add_modifier(Modifier::BOLD)
                    ).alignment(Alignment::Right)
                );
            }

            break;
        }

        Self {
            msg: Line::from(msg).alignment(Alignment::Left),
            modified_sign: sign
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
