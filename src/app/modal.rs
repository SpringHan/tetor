// Modal

#[derive(Debug, Clone, Copy)]
enum Type {
    Normal,
    Insert
}

#[derive(Debug, Clone)]
pub struct Modal {
    _type: Type
}

impl Default for Modal {
    fn default() -> Self {
        Modal { _type: Type::Normal }
    }
}

impl Modal {
    pub fn switch_normal(&mut self) {
        self._type = Type::Normal;
        // TODO: Change cursor type.
    }

    pub fn switch_insert(&mut self) {
        self._type = Type::Insert;
        // TODO: Change cursor type.
    }
}
