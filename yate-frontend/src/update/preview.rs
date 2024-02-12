use yate_keymap::message::Message;

use crate::{
    layout::AppLayout,
    model::{buffer::BufferLine, Model},
};

use super::{buffer, history, path};

pub fn update(model: &mut Model, layout: &AppLayout, message: &Message) {
    let target = &model.preview.path;
    let buffer = &mut model.preview.buffer;
    let layout = &layout.preview;

    super::set_viewport_dimensions(&mut buffer.view_port, layout);

    let lines = if target == &model.current.path {
        Vec::new()
    } else if !target.exists() {
        Vec::new()
    } else if target.is_dir() {
        match path::get_directory_content(&target) {
            Ok(content) => content,
            Err(_) => {
                vec![BufferLine {
                    content: "Error reading directory".to_string(),
                    ..Default::default()
                }]
            }
        }
    } else {
        // TODO: add file preview
        Vec::new()
    };

    buffer::set_content(&model.mode, buffer, lines);
    super::directory::sort_content(&model.mode, buffer);

    buffer::update(&model.mode, buffer, message);

    if !history::set_cursor_index(&target, &model.history, buffer) {
        buffer.cursor = None;
    };
}
