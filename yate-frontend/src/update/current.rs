use yate_keymap::message::Message;

use crate::{layout::AppLayout, model::Model};

use super::path;

pub fn update(model: &mut Model, layout: &AppLayout, message: &Message) {
    path::update_buffer_with_path(
        &mut model.current_directory,
        &layout.current_directory,
        message,
        &model.current_path,
    );
}
