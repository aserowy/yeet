use ratatui::layout::{Constraint, Direction, Layout, Rect};
use yeet_keymap::message::{Buffer, Message, Mode};

use crate::{
    layout::AppLayout,
    model::{buffer::BufferLine, Model},
};

use super::buffer::{self, cursor};

pub fn update(model: &mut Model, layout: &AppLayout, message: Option<&Buffer>) {
    let buffer = &mut model.commandline.buffer;
    let layout = &layout.commandline;

    super::set_viewport_dimensions(&mut buffer.view_port, layout);

    // FIX: key_sequence changed not working
    if let Some(message) = message {
        if let Buffer::ChangeMode(from, to) = message {
            if to != &Mode::Command {
                let bl = BufferLine {
                    content: get_content_with_keysequence(
                        &layout,
                        model.mode.to_string().to_uppercase(),
                        model.key_sequence.clone(),
                    ),
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

// TODO: rework layout into model
pub fn get_content_with_keysequence(
    layout: &Rect,
    content: String,
    key_sequence: String,
) -> String {
    let sequence_len = key_sequence.chars().count() as u16;
    let _layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(100),
            Constraint::Length(sequence_len),
        ])
        .split(layout.clone());

    format!("--{}--{}", content, key_sequence)
}

pub fn height(model: &Model, messages: &Vec<Message>) -> u16 {
    let mut height = model.commandline.buffer.lines.len() as u16;
    for message in messages {
        if let Message::Print(content) = message {
            if content.len() > 1 {
                height = content.len() as u16 + 1;
            }
        }
    }
    height
}
