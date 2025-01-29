// Frame

use crate::app::App;

use ratatui::{layout::{Constraint, Direction, Layout}, Frame};

pub fn main_frame(frame: &mut Frame, app: &mut App) {
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(98),
            Constraint::Percentage(2)
        ])
        .split(frame.size());

    let file_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage(5),
            Constraint::Percentage(95)
        ])
        .split(main_layout[0]);
}
