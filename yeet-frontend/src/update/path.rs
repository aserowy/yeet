use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use ratatui::style::Color;
use yeet_buffer::{
    message::BufferMessage,
    model::{BufferLine, Mode, StylePartial, StylePartialSpan},
    update::update_buffer,
};

use crate::{action::Action, model::Model};

use super::{
    cursor::get_selected_content_from_buffer,
    history::get_selection_from_history,
    preview::{set_preview_to_selected, validate_preview_viewport},
    sign::{set_sign_if_marked, set_sign_if_qfix},
};

#[tracing::instrument(skip(model))]
pub fn add_paths(model: &mut Model, paths: &[PathBuf]) -> Vec<Action> {
    let mut buffer = vec![(
        model.files.current.path.as_path(),
        &mut model.files.current.buffer,
        model.mode == Mode::Navigation,
    )];

    if let Some(preview) = &model.files.preview.path {
        buffer.push((preview, &mut model.files.preview.buffer, preview.is_dir()));
    }

    if let Some(parent) = &model.files.parent.path {
        buffer.push((parent, &mut model.files.parent.buffer, true));
    }

    for (path, buffer, sort) in buffer {
        let paths_for_buffer: Vec<_> = paths.iter().filter(|p| p.parent() == Some(path)).collect();
        if paths_for_buffer.is_empty() {
            continue;
        }

        let mut selection = get_selected_content_from_buffer(buffer);

        let indexes = buffer
            .lines
            .iter()
            .enumerate()
            .map(|(i, bl)| {
                let key = if bl.content.contains('/') {
                    bl.content.split('/').collect::<Vec<_>>()[0].to_string()
                } else {
                    bl.content.clone()
                };

                (key, i)
            })
            .collect::<HashMap<_, _>>();

        for path in paths_for_buffer {
            if let Some(basename) = path.file_name().and_then(|oss| oss.to_str()) {
                let mut line = from(path);
                set_sign_if_marked(&model.marks, &mut line, path);
                set_sign_if_qfix(&model.qfix, &mut line, path);

                if let Some(index) = indexes.get(basename) {
                    buffer.lines[*index] = line;
                } else {
                    buffer.lines.push(line);
                }

                selection = selection.map(|sl| {
                    if sl.starts_with(&[basename, "/"].concat()) {
                        basename.to_owned()
                    } else {
                        sl
                    }
                });
            }
        }

        if sort {
            update_buffer(
                &model.mode,
                buffer,
                &BufferMessage::SortContent(super::SORT),
            );
        }

        if let Some(selection) = selection {
            update_buffer(
                &model.mode,
                buffer,
                &BufferMessage::SetCursorToLineContent(selection),
            );
        }
    }

    let mut actions = Vec::new();
    if let Some(path) = set_preview_to_selected(model) {
        validate_preview_viewport(model);

        let selection = get_selection_from_history(&model.history, &path).map(|s| s.to_owned());
        actions.push(Action::Load(path, selection));
    }

    actions
}

fn from(path: &Path) -> BufferLine {
    let content = match path.file_name() {
        Some(content) => content.to_str().unwrap_or("").to_string(),
        None => "".to_string(),
    };

    // TODO: Handle transition states like adding, removing, renaming
    let style = if path.is_dir() {
        let length = content.chars().count();
        vec![StylePartialSpan {
            end: length,
            style: StylePartial::Foreground(Color::LightBlue),
            ..Default::default()
        }]
    } else {
        vec![]
    };

    BufferLine {
        content,
        style,
        ..Default::default()
    }
}

#[tracing::instrument(skip(model))]
pub fn remove_path(model: &mut Model, path: &Path) -> Vec<Action> {
    if path.starts_with(&model.junk.path) {
        model.junk.remove(path);
    }

    let mut buffer = vec![(
        model.files.current.path.as_path(),
        &mut model.files.current.buffer,
    )];

    if let Some(preview) = &model.files.preview.path {
        buffer.push((preview, &mut model.files.preview.buffer));
    }

    if let Some(parent) = &model.files.parent.path {
        buffer.push((parent, &mut model.files.parent.buffer));
    }

    if let Some(parent) = path.parent() {
        if let Some((_, buffer)) = buffer.into_iter().find(|(p, _)| p == &parent) {
            if let Some(basename) = path.file_name().and_then(|oss| oss.to_str()) {
                let index = buffer
                    .lines
                    .iter()
                    .enumerate()
                    .find(|(_, bl)| bl.content == basename)
                    .map(|(i, _)| i);

                if let Some(index) = index {
                    update_buffer(&model.mode, buffer, &BufferMessage::RemoveLine(index));
                }
            }
        }
    }

    let mut actions = Vec::new();
    if let Some(path) = set_preview_to_selected(model) {
        validate_preview_viewport(model);

        let selection = get_selection_from_history(&model.history, &path).map(|s| s.to_owned());
        actions.push(Action::Load(path, selection));
    }

    actions
}
