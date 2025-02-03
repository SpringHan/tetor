// Basic

use crate::{app::App, error::AppResult, ui::ModalType};

pub(super) fn change_modal(app: &mut App, modal: ModalType) -> AppResult<()> {
    if modal == ModalType::Normal {
        app.get_modal().switch_normal();

        return Ok(())
    }

    app.get_modal().switch_insert();

    Ok(())
}
