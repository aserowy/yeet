use yate_keymap::message::Message;

use crate::{layout::AppLayout, model::Model};

use super::{buffer, path};

pub fn update(model: &mut Model, layout: &AppLayout, message: &Message) {
    if let Some(target) = path::get_selected_path(model) {
        let buffer = &mut model.preview;
        let layout = &layout.preview;

        super::set_viewport_dimensions(&mut buffer.view_port, layout);

        buffer.lines = if target.is_dir() {
            path::get_directory_content(&target)
        } else {
            vec![]
        };

        buffer::update(buffer, message);
    }
}
