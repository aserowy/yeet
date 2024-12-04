use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use yeet_buffer::{
    message::BufferMessage,
    model::{ansi::Ansi, BufferLine, Cursor, Mode, TextBuffer},
    update::update_buffer,
};

use crate::{
    action::Action,
    model::{FileTreeBufferSection, FileTreeBufferSectionBuffer, Model},
};

use super::{
    history::get_selection_from_history,
    junkyard::remove_from_junkyard,
    selection,
    sign::{set_sign_if_marked, set_sign_if_qfix},
};

#[tracing::instrument(skip(model))]
pub fn add_paths(model: &mut Model, paths: &[PathBuf]) -> Vec<Action> {
    let mut buffer_contents = vec![(
        model.files.current.path.as_path(),
        &mut model.files.current_vp,
        &mut model.files.current_cursor,
        &mut model.files.current.buffer,
        model.mode == Mode::Navigation,
    )];

    if let FileTreeBufferSectionBuffer::Text(path, buffer) = &mut model.files.parent {
        buffer_contents.push((
            path.as_path(),
            &mut model.files.parent_vp,
            &mut model.files.parent_cursor,
            buffer,
            path.is_dir(),
        ));
    }

    if let FileTreeBufferSectionBuffer::Text(path, buffer) = &mut model.files.preview {
        buffer_contents.push((
            path.as_path(),
            &mut model.files.preview_vp,
            &mut model.files.preview_cursor,
            buffer,
            path.is_dir(),
        ));
    }

    for (path, viewport, cursor, buffer, sort) in buffer_contents {
        let paths_for_buffer: Vec<_> = paths.iter().filter(|p| p.parent() == Some(path)).collect();
        if paths_for_buffer.is_empty() {
            continue;
        }

        let mut selection = match cursor {
            Some(it) => get_selected_content_from_buffer(it, buffer),
            None => None,
        };

        let indexes = buffer
            .lines
            .iter()
            .enumerate()
            .map(|(i, bl)| {
                let content = bl.content.to_stripped_string();
                let key = if content.contains('/') {
                    content.split('/').collect::<Vec<_>>()[0].to_string()
                } else {
                    content.clone()
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
                viewport,
                cursor,
                &model.mode,
                buffer,
                &BufferMessage::SortContent(super::SORT),
            );
        }

        if let Some(selection) = selection {
            update_buffer(
                viewport,
                cursor,
                &model.mode,
                buffer,
                &BufferMessage::SetCursorToLineContent(selection),
            );
        }
    }

    let mut actions = Vec::new();
    if let Some(path) = selection::get_current_selected_path(model) {
        let selection = get_selection_from_history(&model.history, &path).map(|s| s.to_owned());
        actions.push(Action::Load(
            FileTreeBufferSection::Preview,
            path,
            selection,
        ));
    }

    actions
}

fn get_selected_content_from_buffer(cursor: &Cursor, model: &TextBuffer) -> Option<String> {
    model
        .lines
        .get(cursor.vertical_index)
        .map(|line| line.content.to_stripped_string())
}

fn from(path: &Path) -> BufferLine {
    let content = match path.file_name() {
        Some(content) => content.to_str().unwrap_or("").to_string(),
        None => "".to_string(),
    };

    let content = if path.is_dir() {
        format!("\x1b[94m{}\x1b[39m", content)
    } else {
        content
    };

    BufferLine {
        content: Ansi::new(&content),
        ..Default::default()
    }
}

#[tracing::instrument(skip(model))]
pub fn remove_path(model: &mut Model, path: &Path) -> Vec<Action> {
    if path.starts_with(&model.junk.path) {
        remove_from_junkyard(&mut model.junk, path);
    }

    let current_selection = match &model.files.current_cursor {
        Some(it) => get_selected_content_from_buffer(it, &model.files.current.buffer),
        None => None,
    };

    let mut buffer_contents = vec![(
        model.files.current.path.as_path(),
        &mut model.files.current_vp,
        &mut model.files.current_cursor,
        &mut model.files.current.buffer,
    )];

    if let FileTreeBufferSectionBuffer::Text(path, buffer) = &mut model.files.parent {
        buffer_contents.push((
            path.as_path(),
            &mut model.files.parent_vp,
            &mut model.files.parent_cursor,
            buffer,
        ));
    }

    if let FileTreeBufferSectionBuffer::Text(path, buffer) = &mut model.files.preview {
        buffer_contents.push((
            path.as_path(),
            &mut model.files.preview_vp,
            &mut model.files.preview_cursor,
            buffer,
        ));
    }

    if let Some(parent) = path.parent() {
        if let Some((_, viewport, cursor, buffer)) = buffer_contents
            .into_iter()
            .find(|(p, _, _, _)| p == &parent)
        {
            if let Some(basename) = path.file_name().and_then(|oss| oss.to_str()) {
                let index = buffer
                    .lines
                    .iter()
                    .enumerate()
                    .find(|(_, bl)| bl.content.to_stripped_string() == basename)
                    .map(|(i, _)| i);

                if let Some(index) = index {
                    update_buffer(
                        viewport,
                        cursor,
                        &model.mode,
                        buffer,
                        &BufferMessage::RemoveLine(index),
                    );
                }
            }
        }
    }

    if let Some(selection) = current_selection {
        update_buffer(
            &mut model.files.current_vp,
            &mut model.files.current_cursor,
            &model.mode,
            &mut model.files.current.buffer,
            &BufferMessage::SetCursorToLineContent(selection),
        );
    };

    let mut actions = Vec::new();
    if let Some(path) = selection::get_current_selected_path(model) {
        let selection = get_selection_from_history(&model.history, &path).map(|s| s.to_owned());
        actions.push(Action::Load(
            FileTreeBufferSection::Preview,
            path,
            selection,
        ));
    }

    actions
}
