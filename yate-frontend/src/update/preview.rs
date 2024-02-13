use yate_keymap::message::Message;

use crate::{layout::AppLayout, model::Model};

use super::{buffer, history};

pub fn update(model: &mut Model, layout: &AppLayout, message: &Message) {
    let target = &model.preview.path;
    let buffer = &mut model.preview.buffer;
    let layout = &layout.preview;

    super::set_viewport_dimensions(&mut buffer.view_port, layout);

    buffer::update(&model.mode, buffer, message);

    if !history::set_cursor_index(&target, &model.history, buffer) {
        buffer.cursor = None;
    };
}
