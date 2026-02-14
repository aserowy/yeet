use std::path::PathBuf;

use std::path::Path;

use yeet_buffer::model::BufferLine;

use crate::{
    action::Action,
    event::Message,
    model::{register::Register, FileTreeBuffer},
};

pub fn get_current_selected_path(
    buffer: &FileTreeBuffer,
    cursor: Option<&yeet_buffer::model::Cursor>,
) -> Option<PathBuf> {
    get_current_selected_path_with_exists(buffer, cursor, |path| path.exists())
}

pub fn get_current_selected_path_with_exists(
    buffer: &FileTreeBuffer,
    cursor: Option<&yeet_buffer::model::Cursor>,
    exists: impl Fn(&std::path::Path) -> bool,
) -> Option<PathBuf> {
    get_selected_path_with_base(&buffer.current.path, &buffer.current.buffer, cursor, exists)
}

pub fn get_selected_path_with_base(
    base_path: &Path,
    text_buffer: &yeet_buffer::model::TextBuffer,
    cursor: Option<&yeet_buffer::model::Cursor>,
    exists: impl Fn(&std::path::Path) -> bool,
) -> Option<PathBuf> {
    if text_buffer.lines.is_empty() {
        return None;
    }

    let cursor = cursor?;
    let current = &text_buffer.lines.get(cursor.vertical_index)?;
    if current.content.is_empty() {
        return None;
    }

    let target = base_path.join(current.content.to_stripped_string());

    if exists(&target) {
        Some(target)
    } else {
        None
    }
}

#[allow(dead_code)]
pub fn get_current_selected_bufferline<'a>(
    buffer: &'a mut FileTreeBuffer,
    cursor: Option<&'a yeet_buffer::model::Cursor>,
) -> Option<&'a mut BufferLine> {
    let current_buffer = &mut buffer.current.buffer;
    if current_buffer.lines.is_empty() {
        return None;
    }

    let cursor = cursor?;
    current_buffer.lines.get_mut(cursor.vertical_index)
}

pub fn copy_to_clipboard(
    register: &mut Register,
    buffer: &FileTreeBuffer,
    cursor: Option<&yeet_buffer::model::Cursor>,
) -> Vec<Action> {
    if let Some(path) = get_current_selected_path(buffer, cursor) {
        if let Some(clipboard) = register.clipboard.as_mut() {
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
