use yeet_keymap::message::{Buffer, Message, Mode};

use crate::{
    layout::CommandLineLayout,
    model::{buffer::BufferLine, Model},
};

use super::buffer::{self, cursor};

pub fn update(model: &mut Model, message: Option<&Buffer>) {
    let commandline = &mut model.commandline;
    let buffer = &mut commandline.buffer;

    let sequence_len = model.key_sequence.chars().count() as u16;
    commandline.layout = CommandLineLayout::new(model.layout.commandline, sequence_len);

    super::set_viewport_dimensions(&mut buffer.view_port, &commandline.layout.buffer);

    if let Some(message) = message {
        if let Buffer::ChangeMode(from, to) = message {
            if to != &Mode::Command {
                let bl = BufferLine {
                    content: format!("--{}--", model.mode.to_string().to_uppercase()),
                    ..Default::default()
                };

                if buffer.lines.is_empty() {
                    buffer.lines.push(bl);
                } else {
                    let last = buffer.lines.len() - 1;
                    buffer.lines[last] = bl;
                }

                cursor::validate(to, buffer);
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
}

pub fn height(model: &Model, messages: &Vec<Message>) -> u16 {
    let lines_len = model.commandline.buffer.lines.len();
    let mut height = if lines_len == 0 { 1 } else { lines_len as u16 };
    for message in messages {
        if let Message::Print(content) = message {
            if content.len() > 1 {
                height = content.len() as u16 + 1;
            }
        }
    }
    height
}
