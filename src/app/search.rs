// Search feature

#[derive(Debug, Default)]
pub struct SearchIndicates {
    target_str: String,
    indicates: Vec<(u16, u16)>,

    pub selected: Option<usize>,
}

impl SearchIndicates {
    pub fn clear(&mut self) {
        self.target_str.clear();
        self.selected = None;
        self.indicates.clear();
    }

    pub fn has_history(&self) -> bool {
        if &self.target_str == "" &&
            self.indicates.is_empty() &&
            self.selected.is_none()
        {
            return false
        }

        true
    }

    pub fn indicates(&self) -> &Vec<(u16, u16)> {
        &self.indicates
    }

    pub fn current_indicate(&self) -> Option<(u16, u16)> {
        if self.selected.is_none() {
            return None
        }

        Some(self.indicates[self.selected.unwrap()])
    }

    pub fn replace<I>(&mut self, target: String, iter: I)
    where I: Iterator<Item = (u16, u16)>
    {
        self.target_str = target;
        self.indicates.extend(iter);
    }

    pub fn target_len(&self) -> u16 {
        self.target_str.len() as u16
    }
}
