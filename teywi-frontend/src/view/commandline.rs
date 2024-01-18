use ratatui::{
    prelude::{Constraint, Direction, Layout, Rect},
    text::Line,
    widgets::Paragraph,
    Frame,
};
use teywi_keymap::action::Mode;

use crate::model::Model;

pub fn view(model: &mut Model, frame: &mut Frame, rect: Rect) {
    if model.mode == Mode::Command {
    } else {
        let key_sequence = model.key_sequence.clone();
        let sequence_len = key_sequence.chars().count() as u16;

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(1), Constraint::Max(sequence_len)])
            .split(rect);

        let line: Line = vec![
            "--".into(),
            model.mode.to_string().to_uppercase().into(),
            "--".into(),
        ]
        .into();

        frame.render_widget(Paragraph::new(line), layout[0]);
        frame.render_widget(Paragraph::new(model.key_sequence.clone()), layout[1]);
    }
}
