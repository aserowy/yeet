use ratatui::{widgets::Paragraph, Frame};
use yeet_buffer::view;

use crate::model::Model;

pub fn view(model: &Model, frame: &mut Frame) {
    let commandline = &model.commandline;

    view::view(
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
