use std::path::PathBuf;

use ratatui::style::Color;
use yeet_keymap::message::{ContentKind, Mode};

use crate::{
    action::Action,
    model::{
        buffer::{BufferLine, StylePartial},
        Model,
    },
};

use super::{buffer, directory, history, preview};

pub fn changed(
    model: &mut Model,
    path: &PathBuf,
    contents: &[(ContentKind, String)],
) -> Option<Vec<Action>> {
    // TODO: handle unsaved changes
    let mut buffer = vec![
        (model.current.path.as_path(), &mut model.current.buffer),
        (model.preview.path.as_path(), &mut model.preview.buffer),
    ];

    if let Some(parent) = &model.parent.path {
        buffer.push((parent, &mut model.parent.buffer));
    }

    let mut actions = Vec::new();
    if let Some((_, buffer)) = buffer.into_iter().find(|(p, _)| p == path) {
        let content = contents
            .iter()
            .map(|(knd, cntnt)| get_bufferline_by_enumeration_content(knd, cntnt))
            .collect();

        buffer::set_content(&model.mode, buffer, content);

        if let Some(preview_actions) = preview::path(model, true, true) {
            actions.extend(preview_actions);
            model.preview.buffer.lines.clear();
            preview::viewport(model);
        }
    }

    if actions.is_empty() {
        None
    } else {
        Some(actions)
    }
}

pub fn finished(model: &mut Model, path: &PathBuf) -> Option<Vec<Action>> {
    if model.mode != Mode::Navigation {
        return None;
    }

    let mut buffer = vec![
        (model.current.path.as_path(), &mut model.current.buffer),
        (model.preview.path.as_path(), &mut model.preview.buffer),
    ];

    if let Some(parent) = &model.parent.path {
        buffer.push((parent, &mut model.parent.buffer));
    }

    let mut actions = Vec::new();
    if let Some((path, buffer)) = buffer.into_iter().find(|(p, _)| p == path) {
        directory::sort_content(&model.mode, buffer);
        history::set_cursor_index(path, &model.history, buffer);

        if let Some(preview_actions) = preview::path(model, true, true) {
            actions.extend(preview_actions);
            model.preview.buffer.lines.clear();
            preview::viewport(model);
        }
    }

    if actions.is_empty() {
        None
    } else {
        Some(actions)
    }
}

fn get_bufferline_by_enumeration_content(kind: &ContentKind, content: &String) -> BufferLine {
    // TODO: refactor with by path
    let style = if kind == &ContentKind::Directory {
        let length = content.chars().count();
        vec![(0, length, StylePartial::Foreground(Color::LightBlue))]
    } else {
        vec![]
    };

    BufferLine {
        content: content.to_string(),
        style,
        ..Default::default()
    }
}
