// Search feature

use std::ops::Range;

#[derive(Debug, Default)]
pub struct SearchIndicates {
    target_str: String,
    indicates: Vec<(Range<u16>, u16)>,

    selected: Option<usize>,
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

    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn selected_mut(&mut self) -> &mut Option<usize> {
        &mut self.selected
    }

    pub fn indicates(&self) -> &Vec<(Range<u16>, u16)> {
        &self.indicates
    }

    pub fn indicates_find(&self, cursor: (u16, u16)) -> bool {
        for (x_range, y) in self.indicates.iter() {
            if cursor.1 == *y && x_range.contains(&cursor.0) {
                return true
            }
        }

        false
    }

    pub fn current_indicate(&self) -> Option<(u16, u16)> {
        if self.selected.is_none() {
            return None
        }

        let current_raw = self.indicates[self.selected.unwrap()].to_owned();
        Some((current_raw.0.start, current_raw.1))
    }

    pub fn set<I>(&mut self, target: String, iter: I)
    where I: Iterator<Item = (u16, u16)>
    {
        self.target_str = target;
        self.selected = None;

        let target_len = self.target_str.len() as u16;
        let to_raw = iter.map(|indicate| {
            ((indicate.0 .. indicate.0 + target_len), indicate.1)
        }).collect::<Vec<_>>();

        self.indicates.extend(to_raw.into_iter());
    }
}
