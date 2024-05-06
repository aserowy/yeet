use std::path::Path;

use yeet_buffer::model::{BufferLine, Sign, SignIdentifier};

use crate::model::Model;

pub fn set_sign(bl: &mut BufferLine, sign: Sign) {
    let is_signed = bl.signs.iter().any(|s| s.id == sign.id);
    if is_signed {
        return;
    }
    bl.signs.push(sign);
}

pub fn unset_sign_on_all_buffers(model: &mut Model, sign_id: SignIdentifier) {
    let all_buffer = model.files.get_mut_directories();
    for (_, _, buffer) in all_buffer {
        for line in &mut buffer.lines {
            unset_sign(line, sign_id);
        }
    }
}

pub fn unset_sign_for_path(model: &mut Model, path: &Path, sign_id: SignIdentifier) {
    let parent = match path.parent() {
        Some(it) => it,
        None => return,
    };

    let lines = if parent == model.files.current.path {
        &mut model.files.current.buffer.lines
    } else if Some(parent) == model.files.preview.path.as_deref() {
        &mut model.files.preview.buffer.lines
    } else if Some(parent) == model.files.parent.path.as_deref() {
        &mut model.files.parent.buffer.lines
    } else {
        return;
    };

    let file_name = match path.file_name() {
        Some(it) => match it.to_str() {
            Some(it) => it,
            None => return,
        },
        None => return,
    };

    if let Some(line) = lines.iter_mut().find(|bl| bl.content == file_name) {
        unset_sign(line, sign_id);
    }
}

pub fn unset_sign(bl: &mut BufferLine, sign_id: SignIdentifier) {
    let position = bl.signs.iter().position(|s| s.id == sign_id);
    if let Some(position) = position {
        bl.signs.remove(position);
    }
}
