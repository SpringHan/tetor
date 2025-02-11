// Info Line

use ratatui::{style::{Color, Modifier, Style}, text::Line, widgets::Widget};

use crate::{app::App, command::CommandPrior, ui::ModalType};

#[derive(Debug, Clone)]
pub struct InfoLine<'a> {
    msg: Line<'a>
}

impl<'a> From<&mut App> for InfoLine<'a> {
    fn from(app: &mut App) -> Self {
        let mut msg = String::new();
        let mut style = Style::default();

        // TODO: Display for file modification case
        loop {
            if !app.app_errors.empty() {
                app.prior_command = CommandPrior::ConfirmError;
                msg = app.app_errors.get_first();
                style.fg = Some(Color::Red);
                break;
            }

            // TODO: Add search input before modal

            if app.editor_state.modal.modal() == ModalType::Insert {
                msg = String::from(" --INSERT--");
                style = style.add_modifier(Modifier::BOLD);
            }

            break;
        }

        Self { msg: Line::styled(msg, style) }
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
    }
}
