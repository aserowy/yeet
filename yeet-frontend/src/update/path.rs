use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use yeet_buffer::{message::BufferMessage, model::Mode, update};

use crate::{action::Action, model::Model};

use super::{bufferline, cursor, mark, preview, qfix};

#[tracing::instrument(skip(model))]
pub fn add(model: &mut Model, paths: &[PathBuf]) -> Vec<Action> {
    add_paths(model, paths);

    let mut actions = Vec::new();
    if let Some(path) = preview::selected_path(model) {
        preview::viewport(model);

        let selection = model.history.get_selection(&path).map(|s| s.to_owned());
        actions.push(Action::Load(path, selection));
    }

    actions
}

fn add_paths(model: &mut Model, paths: &[PathBuf]) {
    let mut buffer = vec![(
        model.file_buffer.current.path.as_path(),
        &mut model.file_buffer.current.buffer,
        model.mode == Mode::Navigation,
    )];

    if let Some(preview) = &model.file_buffer.preview.path {
        buffer.push((
            preview,
            &mut model.file_buffer.preview.buffer,
            preview.is_dir(),
        ));
    }

    if let Some(parent) = &model.file_buffer.parent.path {
        buffer.push((parent, &mut model.file_buffer.parent.buffer, true));
    }

    for (path, buffer, sort) in buffer {
        let paths_for_buffer: Vec<_> = paths.iter().filter(|p| p.parent() == Some(path)).collect();
        if paths_for_buffer.is_empty() {
            continue;
        }

        let selection = cursor::get_selection(buffer);

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
                let mut line = bufferline::from(path);
                mark::set_sign_if_marked(&model.marks, &mut line, path);
                qfix::set_sign_if_qfix(&model.qfix, &mut line, path);

                if let Some(index) = indexes.get(basename) {
                    buffer.lines[*index] = line;
                } else {
                    buffer.lines.push(line);
                }
            }
        }

        if sort {
            update::update(
                &model.mode,
                &model.search,
                buffer,
                &BufferMessage::SortContent(super::SORT),
            );
        }

        if let Some(selection) = selection {
            update::update(
                &model.mode,
                &model.search,
                buffer,
                &BufferMessage::SetCursorToLineContent(selection),
            );
        }
    }
}

#[tracing::instrument(skip(model))]
pub fn remove(model: &mut Model, path: &Path) -> Vec<Action> {
    remove_path(model, path);

    let mut actions = Vec::new();
    if let Some(path) = preview::selected_path(model) {
        preview::viewport(model);

        let selection = model.history.get_selection(&path).map(|s| s.to_owned());
        actions.push(Action::Load(path, selection));
    }

    actions
}

fn remove_path(model: &mut Model, path: &Path) {
    let mut buffer = vec![(
        model.file_buffer.current.path.as_path(),
        &mut model.file_buffer.current.buffer,
    )];

    if let Some(preview) = &model.file_buffer.preview.path {
        buffer.push((preview, &mut model.file_buffer.preview.buffer));
    }

    if let Some(parent) = &model.file_buffer.parent.path {
        buffer.push((parent, &mut model.file_buffer.parent.buffer));
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
                    update::update(
                        &model.mode,
                        &model.search,
                        buffer,
                        &BufferMessage::RemoveLine(index),
                    );
                }
            }
        }
    }
}
