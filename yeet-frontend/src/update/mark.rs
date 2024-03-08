use std::path::Path;

use ratatui::style::Color;

use crate::model::{
    buffer::{BufferLine, Sign, StylePartial, StylePartialSpan},
    mark::Marks,
    Model,
};

pub fn print(marks: &Marks) -> Vec<String> {
    let mut marks: Vec<_> = marks
        .entries
        .iter()
        .map(|(key, path)| (key, path.to_string_lossy().to_string()))
        .map(|(key, path)| format!("{:<4} {}", key, path))
        .collect();

    marks.sort();

    let mut contents = vec![":marks".to_string(), "Char Content".to_string()];
    contents.extend(marks);

    contents
}

pub fn set_sign(marks: &Marks, bl: &mut BufferLine, path: &Path) {
    let is_marked = marks.entries.values().any(|p| p == path);
    if !is_marked {
        return;
    }

    let sign = 'm';
    let is_signed = bl.signs.iter().any(|s| s.content == sign);
    if is_signed {
        return;
    }

    bl.signs.push(Sign {
        content: sign,
        priority: 0,
        style: vec![StylePartialSpan {
            start: 0,
            end: 1,
            style: StylePartial::Foreground(Color::Yellow),
        }],
    });
}

pub fn unset_sign(model: &mut Model, removed: &Path) {
    let parent = match removed.parent() {
        Some(it) => it,
        None => return,
    };

    let lines = if parent == &model.current.path {
        &mut model.current.buffer.lines
    } else if parent == &model.preview.path {
        &mut model.preview.buffer.lines
    } else if Some(parent) == model.parent.path.as_ref().map(|p| p.as_path()) {
        &mut model.parent.buffer.lines
    } else {
        return;
    };

    let file_name = match removed.file_name() {
        Some(it) => match it.to_str() {
            Some(it) => it,
            None => return,
        },
        None => return,
    };

    for line in lines {
        if line.content == file_name {
            let position = line.signs.iter().position(|s| s.content == 'm');
            if let Some(position) = position {
                line.signs.remove(position);
            }
        }
    }
}
