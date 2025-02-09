// Modal

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalType {
    Normal,
    Insert
}

#[derive(Debug, Clone, Copy)]
pub struct Modal {
    _type: ModalType
}

impl Default for Modal {
    fn default() -> Self {
        Modal { _type: ModalType::Normal }
    }
}

impl Modal {
    pub fn modal(&self) -> ModalType {
        self._type
    }

    pub fn switch_normal(&mut self) {
        self._type = ModalType::Normal;
        // TODO: Change cursor type.
    }

    pub fn switch_insert(&mut self) {
        self._type = ModalType::Insert;
        // TODO: Change cursor type.
    }
}
