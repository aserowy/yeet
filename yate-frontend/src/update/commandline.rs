use yate_keymap::message::{Message, Mode};

use crate::{
    layout::AppLayout,
    model::{buffer::BufferLine, Model},
};

use super::buffer;

pub fn update(model: &mut Model, layout: &AppLayout, message: &Message) {
    let buffer = &mut model.commandline;
    let layout = &layout.commandline;

    buffer.view_port.height = usize::from(layout.height);
    buffer.view_port.width = usize::from(layout.width);

    if let Message::ChangeMode(from, to) = message {
        if from == &Mode::Command && to != &Mode::Command {
            buffer.lines = vec![BufferLine::default()];
        }

        if from != &Mode::Command && to == &Mode::Command {
            buffer::reset_view(&mut buffer.view_port, &mut buffer.cursor);

            let bufferline = BufferLine {
                prefix: Some(":".to_string()),
                ..Default::default()
            };

            buffer.lines = vec![bufferline];
        }
    }

    buffer::update(buffer, message);
}
