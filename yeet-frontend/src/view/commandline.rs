use ratatui::{widgets::Paragraph, Frame};

use crate::model::Model;

use super::buffer;

pub fn view(model: &Model, frame: &mut Frame) {
    let commandline = &model.commandline;

    buffer::view(
        &model.mode,
        &commandline.buffer,
        frame,
        commandline.layout.buffer,
    );

    frame.render_widget(
        Paragraph::new(model.key_sequence.clone()),
        commandline.layout.key_sequence,
    );
}
