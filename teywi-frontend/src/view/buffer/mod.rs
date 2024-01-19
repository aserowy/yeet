use ratatui::{prelude::Rect, widgets::Paragraph, Frame};

use crate::model::Buffer;

pub fn view(buffer: &Buffer, frame: &mut Frame, rect: Rect) {
    let text = buffer.lines.join("\n");

    frame.render_widget(Paragraph::new(text), rect);
}
