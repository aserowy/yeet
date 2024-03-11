use std::path::Path;

use ratatui::style::Color;

use crate::model::{
    buffer::{BufferLine, Sign, SignIdentifier, StylePartial},
    qfix::QuickFix,
    Model,
};

use super::model::current;

pub fn toggle(model: &mut Model) {
    let selected = current::selection(model);
    if let Some(selected) = selected {
        if model.qfix.entries.contains(&selected) {
            model.qfix.entries.remove(&selected);
            if let Some(bl) = current::selected_bufferline(model) {
                unset_sign(bl);
            }
        } else {
            model.qfix.entries.insert(selected);
            if let Some(bl) = current::selected_bufferline(model) {
                set_sign(bl);
            }
        }
    }
}

pub fn print(qfix: &QuickFix) -> Vec<String> {
    let max_width = (qfix.entries.len() + 1).to_string().len();

    let entries: Vec<_> = qfix
        .entries
        .iter()
        .enumerate()
        .map(|(i, path)| (i + 1, path.to_string_lossy().to_string()))
        .map(|(i, path)| format!("{:>max_width$} {}", i, path))
        .collect();

    let mut contents = vec![":cl".to_string()];
    contents.extend(entries);

    contents
}

pub fn set_sign_if_qfix(qfix: &QuickFix, bl: &mut BufferLine, path: &Path) {
    let is_marked = qfix.entries.iter().any(|p| p == path);
    if !is_marked {
        return;
    }

    set_sign(bl);
}

// TODO: refactor with marks impl
fn set_sign(bl: &mut BufferLine) {
    let sign = 'c';
    let is_signed = bl.signs.iter().any(|s| s.id == SignIdentifier::QuickFix);
    if is_signed {
        return;
    }

    bl.signs.push(Sign {
        id: SignIdentifier::QuickFix,
        content: sign,
        priority: 0,
        style: vec![StylePartial::Foreground(Color::LightMagenta)],
    });
}

// TODO: refactor with marks impl
fn unset_sign(bl: &mut BufferLine) {
    let position = bl
        .signs
        .iter()
        .position(|s| s.id == SignIdentifier::QuickFix);

    if let Some(position) = position {
        bl.signs.remove(position);
    }
}
