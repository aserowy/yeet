use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use y1337_keymap::message::Mode;

use crate::model::{buffer::Buffer, Model};

pub fn add_paths(model: &mut Model, paths: &[PathBuf]) {
    let mut buffer = vec![
        (
            model.current.path.as_path(),
            &mut model.current.buffer,
            model.mode == Mode::Navigation,
        ),
        (
            model.preview.path.as_path(),
            &mut model.preview.buffer,
            true,
        ),
    ];

    if let Some(parent) = &model.parent.path {
        buffer.push((parent, &mut model.parent.buffer, true));
    }

    for (path, buffer, sort) in buffer {
        let paths_for_buffer: Vec<_> = paths.iter().filter(|p| p.parent() == Some(path)).collect();
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
                let line = super::path::get_bufferline_by_path(path);
                if let Some(index) = indexes.get(basename) {
                    buffer.lines[*index] = line;
                } else {
                    buffer.lines.push(line);
                }
            }
        }

        if sort {
            sort_content(&model.mode, buffer);
        }

        super::buffer::cursor::validate(&model.mode, buffer);
        // TODO: correct cursor to stay on selection
    }
}

pub fn sort_content(mode: &Mode, model: &mut Buffer) {
    model.lines.sort_unstable_by(|a, b| {
        a.content
            .to_ascii_uppercase()
            .cmp(&b.content.to_ascii_uppercase())
    });
    super::buffer::cursor::validate(mode, model);
}

pub fn remove_path(model: &mut Model, path: &Path) {
    let mut buffer = vec![
        (model.current.path.as_path(), &mut model.current.buffer),
        (model.preview.path.as_path(), &mut model.preview.buffer),
    ];

    if let Some(parent) = &model.parent.path {
        buffer.push((parent, &mut model.parent.buffer));
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
                    buffer.lines.remove(index);
                    super::buffer::cursor::validate(&model.mode, buffer);
                }
            }
        }
    }
}
