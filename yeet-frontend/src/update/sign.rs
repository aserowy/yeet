use std::path::Path;
use yeet_buffer::model::{BufferLine, Sign, SignIdentifier};

use crate::model::{
    mark::{Marks, MARK_SIGN_ID},
    qfix::{QuickFix, QFIX_SIGN_ID},
    Model, PreviewContent,
};

pub fn set_sign_if_qfix(qfix: &QuickFix, bl: &mut BufferLine, path: &Path) {
    let is_marked = qfix.entries.iter().any(|p| p == path);
    if !is_marked {
        return;
    }

    set_sign(bl, QFIX_SIGN_ID);
}

pub fn set_sign_if_marked(marks: &Marks, bl: &mut BufferLine, path: &Path) {
    let is_marked = marks.entries.values().any(|p| p == path);
    if !is_marked {
        return;
    }

    set_sign(bl, MARK_SIGN_ID);
}

pub fn set_sign(bl: &mut BufferLine, sign_id: SignIdentifier) {
    let is_signed = bl.signs.iter().any(|s| s.id == sign_id);
    if is_signed {
        return;
    }

    if let Some(sign) = generate_sign(sign_id) {
        bl.signs.push(sign);
    }
}

fn generate_sign(sign_id: SignIdentifier) -> Option<Sign> {
    match sign_id {
        QFIX_SIGN_ID => Some(Sign {
            id: QFIX_SIGN_ID,
            content: 'c',
            style: "\x1b[1;95m".to_string(),
            priority: 0,
        }),
        MARK_SIGN_ID => Some(Sign {
            id: MARK_SIGN_ID,
            content: 'm',
            style: "\x1b[1;96m".to_string(),
            priority: 0,
        }),
        _ => None,
    }
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
    } else if Some(parent) == model.files.parent.path.as_deref() {
        &mut model.files.parent.buffer.lines
    } else {
        if let PreviewContent::Buffer(dir) = &mut model.files.preview {
            &mut dir.buffer.lines
        } else {
            return;
        }
    };

    let file_name = match path.file_name() {
        Some(it) => match it.to_str() {
            Some(it) => it,
            None => return,
        },
        None => return,
    };

    if let Some(line) = lines
        .iter_mut()
        .find(|bl| bl.content.to_stripped_string() == file_name)
    {
        unset_sign(line, sign_id);
    }
}

pub fn unset_sign(bl: &mut BufferLine, sign_id: SignIdentifier) {
    let position = bl.signs.iter().position(|s| s.id == sign_id);
    if let Some(position) = position {
        bl.signs.remove(position);
    }
}
