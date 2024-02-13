use std::path::Path;

use yate_keymap::message::Mode;

use crate::model::{buffer::Buffer, Model};

pub fn add_path(model: &mut Model, path: &Path) {
    // TODO: refactor into directory mod
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

    if let Some(parent) = path.parent() {
        buffer.push((parent, &mut model.parent.buffer, true));
    }

    if let Some(parent) = path.parent() {
        if let Some((_, buffer, sort)) = buffer.into_iter().find(|(p, _, _)| p == &parent) {
            // TODO: better closer warmer: remove virtual entries... instead of this shiat
            if let Some(basename) = path.file_name().map(|oss| oss.to_str()).flatten() {
                let exists = buffer
                    .lines
                    .iter()
                    .find(|bl| bl.content == basename)
                    .is_some();

                if !exists {
                    buffer.lines.push(super::path::get_bufferline_by_path(path));

                    if path.is_dir() {
                        let basepath = format!("{basename}/");

                        // NOTE: this removes virtual adds like 'dirname/filename'
                        let index = buffer
                            .lines
                            .iter()
                            .enumerate()
                            .find(|(_, bl)| bl.content.starts_with(&basepath))
                            .map(|(i, _)| i);

                        if let Some(index) = index {
                            buffer.lines.remove(index);
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

    if let Some(parent) = path.parent() {
        buffer.push((parent, &mut model.parent.buffer));
    }

    if let Some(parent) = path.parent() {
        if let Some((_, buffer)) = buffer.into_iter().find(|(p, _)| p == &parent) {
            // TODO: better closer warmer: remove virtual entries... instead of this shiat
            if let Some(basename) = path.file_name().map(|oss| oss.to_str()).flatten() {
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
