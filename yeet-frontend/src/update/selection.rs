use std::path::PathBuf;

use yeet_buffer::model::BufferLine;

use crate::model::Model;

pub fn get_current_selected_path(model: &Model) -> Option<PathBuf> {
    let buffer = &model.files.current.buffer;
    if buffer.lines.is_empty() {
        return None;
    }

    let cursor = &buffer.cursor.as_ref()?;
    let current = &buffer.lines.get(cursor.vertical_index)?;
    if current.content.is_empty() {
        return None;
    }

    let target = model.files.current.path.join(&current.content);
    if target.exists() {
        Some(target)
    } else {
        None
    }
}

pub fn get_current_selected_bufferline(model: &mut Model) -> Option<&mut BufferLine> {
    let buffer = &mut model.files.current.buffer;
    if buffer.lines.is_empty() {
        return None;
    }

    let cursor = &buffer.cursor.as_ref()?;

    buffer.lines.get_mut(cursor.vertical_index)
}
