use ratatui::{widgets::Paragraph, Frame};
use yeet_buffer::view;

use crate::model::Model;

pub fn view(model: &Model, frame: &mut Frame) {
    let commandline = &model.commandline;

    view::view(
        &commandline.viewport,
        &commandline.cursor,
        &model.modes.current,
        &commandline.buffer,
        &false,
        frame,
        commandline.layout.buffer,
    );

    frame.render_widget(
        Paragraph::new(model.commandline.key_sequence.clone()),
        commandline.layout.key_sequence,
    );
}
