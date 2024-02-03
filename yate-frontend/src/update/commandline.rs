use yate_keymap::message::{Message, Mode};

use crate::{
    event::PostRenderAction,
    layout::AppLayout,
    model::{buffer::BufferLine, Model},
};

use super::buffer;

pub fn update(
    model: &mut Model,
    layout: &AppLayout,
    message: &Message,
) -> Option<Vec<PostRenderAction>> {
    let buffer = &mut model.commandline;
    let layout = &layout.commandline;

    super::set_viewport_dimensions(&mut buffer.view_port, layout);

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

    buffer::update(&model.mode, buffer, message);

    None
}
