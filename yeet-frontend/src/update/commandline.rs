use yeet_keymap::message::{Buffer, Message, Mode};

use crate::{
    layout::AppLayout,
    model::{buffer::BufferLine, Model},
};

use super::buffer;

pub fn update(model: &mut Model, layout: &AppLayout, message: &Buffer) {
    let buffer = &mut model.commandline;
    let layout = &layout.commandline;

    super::set_viewport_dimensions(&mut buffer.view_port, layout);

    if let Buffer::ChangeMode(from, to) = message {
        if from == &Mode::Command && to != &Mode::Command {
            buffer::set_content(to, buffer, vec![BufferLine::default()]);
        }

        if from != &Mode::Command && to == &Mode::Command {
            buffer::reset_view(buffer);

            let bufferline = BufferLine {
                prefix: Some(":".to_string()),
                ..Default::default()
            };

            buffer::set_content(to, buffer, vec![bufferline]);
        }
    }

    buffer::update(&model.mode, buffer, message);
}

pub fn height(messages: &Vec<Message>) -> u16 {
    let mut height = 1;
    for message in messages {
        if let Message::Print(content) = message {
            if content.len() > 1 {
                height = content.len() as u16 + 1;
            }
        }
    }
    height
}
