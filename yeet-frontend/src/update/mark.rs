use std::path::Path;

use ratatui::style::Color;
use yeet_buffer::model::{BufferLine, Sign, StylePartial};

use crate::{
    action::Action,
    model::{
        mark::{Marks, MARK_SIGN_ID},
        Model,
    },
    task::Task,
};

use super::current;

pub fn add(model: &mut Model, char: char) {
    let selected = current::selection(model);
    if let Some(selected) = selected {
        let removed = model.marks.entries.insert(char, selected);
        if let Some(removed) = removed {
            unset_sign(model, &removed);
        }

        if let Some(bl) = current::selected_bufferline(model) {
            set_sign(bl);
        }
    }
}

pub fn delete(model: &mut Model, delete: &Vec<char>) -> Vec<Action> {
    let mut persisted = Vec::new();
    for mark in delete {
        let deleted = model.marks.entries.remove_entry(mark);
        if let Some((mark, path)) = deleted {
            unset_sign(model, path.as_path());
            persisted.push(mark);
        }
    }

    if persisted.is_empty() {
        Vec::new()
    } else {
        vec![Action::Task(Task::DeleteMarks(persisted))]
    }
}

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

pub fn set_sign_if_marked(marks: &Marks, bl: &mut BufferLine, path: &Path) {
    let is_marked = marks.entries.values().any(|p| p == path);
    if !is_marked {
        return;
    }

    set_sign(bl);
}

fn set_sign(bl: &mut BufferLine) {
    let is_signed = bl.signs.iter().any(|s| s.id == MARK_SIGN_ID);
    if is_signed {
        return;
    }

    bl.signs.push(Sign {
        id: MARK_SIGN_ID,
        content: 'm',
        priority: 0,
        style: vec![StylePartial::Foreground(Color::LightMagenta)],
    });
}

fn unset_sign(model: &mut Model, removed: &Path) {
    let parent = match removed.parent() {
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

    let file_name = match removed.file_name() {
        Some(it) => match it.to_str() {
            Some(it) => it,
            None => return,
        },
        None => return,
    };

    if let Some(line) = lines.iter_mut().find(|bl| bl.content == file_name) {
        let position = line.signs.iter().position(|s| s.id == MARK_SIGN_ID);
        if let Some(position) = position {
            line.signs.remove(position);
        }
    }
}
