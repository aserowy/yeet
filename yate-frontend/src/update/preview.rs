use yate_keymap::message::Message;

use crate::{layout::AppLayout, model::Model};

use super::path;

pub fn update(model: &mut Model, layout: &AppLayout, message: &Message) {
    if let Some(target) = path::get_target_path(model) {
        path::update_buffer_with_path(&mut model.preview, &layout.preview, message, &target);
    }
}
