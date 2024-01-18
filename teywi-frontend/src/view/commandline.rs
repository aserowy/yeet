use ratatui::{prelude::Rect, text::Line, widgets::Paragraph, Frame};
use teywi_keymap::action::Mode;

use crate::model::Model;

pub fn view(model: &mut Model, frame: &mut Frame, rect: Rect) {
    if model.mode == Mode::Command {
    } else {
        let line: Line = vec![
            "--".into(),
            model.mode.to_string().to_uppercase().into(),
            "--".into(),
        ]
        .into();

        frame.render_widget(Paragraph::new(line), rect)
    }
}
