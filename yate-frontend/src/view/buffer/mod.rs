use ratatui::{prelude::Rect, widgets::Paragraph, Frame};
use yate_keymap::message::Mode;

use crate::model::buffer::Buffer;

mod style;

pub fn view(mode: &Mode, model: &Buffer, frame: &mut Frame, rect: Rect) {
    let rendered = get_rendered_lines(model);
    let styled = style::get_styled_lines(&model.view_port, mode, &model.cursor, &rendered);

    frame.render_widget(Paragraph::new(styled), rect);
}

pub fn get_rendered_lines(model: &Buffer) -> Vec<String> {
    model
        .lines
        .iter()
        .skip(model.view_port.vertical_index)
        .take(model.view_port.height)
        .map(|line| line.to_owned())
        .collect()
}
