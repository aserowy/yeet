use yate_keymap::message::Message;

use crate::{
    layout::AppLayout,
    model::{buffer::BufferLine, Model},
};

use super::{buffer, history, path};

pub fn update(model: &mut Model, layout: &AppLayout, message: &Message) {
    if let Some(target) = path::get_selected_path(model) {
        let buffer = &mut model.preview;
        let layout = &layout.preview;

        super::set_viewport_dimensions(&mut buffer.view_port, layout);

        buffer.lines = if target == model.current_path {
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

        buffer::update(&model.mode, buffer, message);

        if !history::set_cursor_index(&target, &model.history, buffer) {
            buffer.cursor = None;
        };
    }
}
