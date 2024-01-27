use yate_keymap::message::Message;

use crate::{layout::AppLayout, model::Model};

use super::{buffer, path};

pub fn update(model: &mut Model, layout: &AppLayout, message: &Message) {
    let buffer = &mut model.current_directory;
    let layout = &layout.current_directory;

    super::set_viewport_dimensions(&mut buffer.view_port, layout);
    buffer.lines = path::get_directory_content(&model.current_path);

    buffer::update(buffer, message);
}
