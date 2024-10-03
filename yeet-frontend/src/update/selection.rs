use std::path::PathBuf;

use yeet_buffer::model::BufferLine;

use crate::{action::Action, event::Message, model::Model};

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

    let target = model
        .files
        .current
        .path
        .join(current.content.to_stripped_string());

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

pub fn copy_current_selected_path_to_clipboard(model: &mut Model) -> Vec<Action> {
    if let Some(path) = get_current_selected_path(model) {
        if let Some(clipboard) = model.register.clipboard.as_mut() {
            match clipboard.set_text(path.to_string_lossy()) {
                Ok(_) => Vec::new(),
                Err(err) => vec![Action::EmitMessages(vec![Message::Error(err.to_string())])],
            }
        } else {
            vec![Action::EmitMessages(vec![Message::Error(
                "Clipboard not available".to_string(),
            )])]
        }
    } else {
        Vec::new()
    }
}
