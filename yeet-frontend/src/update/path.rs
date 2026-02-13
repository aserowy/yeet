use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use yeet_buffer::{
    message::BufferMessage,
    model::{ansi::Ansi, BufferLine, Cursor, Mode, TextBuffer},
};

use crate::{
    action::Action,
    model::{
        history::History, junkyard::JunkYard, mark::Marks, qfix::QuickFix, Buffer,
        FileTreeBufferSection, FileTreeBufferSectionBuffer,
    },
};

use super::{
    history::get_selection_from_history,
    junkyard::remove_from_junkyard,
    selection,
    sign::{set_sign_if_marked, set_sign_if_qfix},
};

#[tracing::instrument(skip(buffers))]
pub fn add(
    history: &History,
    marks: &Marks,
    qfix: &QuickFix,
    mode: &Mode,
    buffers: Vec<&mut Buffer>,
    paths: &[PathBuf],
) -> Vec<Action> {
    let mut actions = Vec::new();
    for buffer in buffers {
        let buffer = match buffer {
            Buffer::FileTree(it) => it,
            _ => continue,
        };

        if mode == &Mode::Navigation {
            if let Some(cursor) = buffer.parent_cursor.as_mut() {
                apply_paths_to_buffer(
                    buffer.current.path.as_path(),
                    cursor,
                    &mut buffer.current.buffer,
                    true,
                    paths,
                    marks,
                    qfix,
                    mode,
                );
            }
        }

        if let (Some(cursor), FileTreeBufferSectionBuffer::Text(path, text_buffer)) =
            (buffer.parent_cursor.as_mut(), &mut buffer.parent)
        {
            apply_paths_to_buffer(
                path.as_path(),
                cursor,
                text_buffer,
                path.is_dir(),
                paths,
                marks,
                qfix,
                mode,
            );
        }

        if let (Some(cursor), FileTreeBufferSectionBuffer::Text(path, text_buffer)) =
            (buffer.preview_cursor.as_mut(), &mut buffer.preview)
        {
            apply_paths_to_buffer(
                path.as_path(),
                cursor,
                text_buffer,
                path.is_dir(),
                paths,
                marks,
                qfix,
                mode,
            );
        }

        let _ = (marks, qfix, mode);

        if let Some(path) =
            selection::get_current_selected_path(buffer, buffer.parent_cursor.as_ref())
        {
            let selection = get_selection_from_history(history, &path).map(|s| s.to_owned());
            actions.push(Action::Load(
                FileTreeBufferSection::Preview,
                path,
                selection,
            ));
        }
    }

    actions
}

fn get_selected_content_from_buffer(cursor: &Cursor, model: &TextBuffer) -> Option<String> {
    model
        .lines
        .get(cursor.vertical_index)
        .map(|line| line.content.to_stripped_string())
}

fn apply_paths_to_buffer(
    dir_path: &Path,
    cursor: &mut Cursor,
    buffer: &mut TextBuffer,
    sort: bool,
    paths: &[PathBuf],
    marks: &Marks,
    qfix: &QuickFix,
    mode: &Mode,
) {
    let paths_for_buffer: Vec<_> = paths
        .iter()
        .filter(|p| p.parent() == Some(dir_path))
        .collect();
    if paths_for_buffer.is_empty() {
        return;
    }

    let mut selection = get_selected_content_from_buffer(cursor, buffer);

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
            set_sign_if_marked(marks, &mut line, path);
            set_sign_if_qfix(qfix, &mut line, path);

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
        let message = BufferMessage::SortContent(super::SORT);
        yeet_buffer::update(
            None,
            Some(cursor),
            mode,
            buffer,
            std::slice::from_ref(&message),
        );
    }

    if let Some(selection) = selection {
        let message = BufferMessage::SetCursorToLineContent(selection);
        yeet_buffer::update(
            None,
            Some(cursor),
            mode,
            buffer,
            std::slice::from_ref(&message),
        );
    }
}

fn remove_line_from_buffer(cursor: &mut Cursor, mode: &Mode, buffer: &mut TextBuffer, path: &Path) {
    if let Some(basename) = path.file_name().and_then(|oss| oss.to_str()) {
        let index = buffer
            .lines
            .iter()
            .enumerate()
            .find(|(_, bl)| bl.content.to_stripped_string() == basename)
            .map(|(i, _)| i);

        if let Some(index) = index {
            let message = BufferMessage::RemoveLine(index);
            yeet_buffer::update(
                None,
                Some(cursor),
                mode,
                buffer,
                std::slice::from_ref(&message),
            );
        }
    }
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

#[tracing::instrument(skip(junk, buffers))]
pub fn remove(
    history: &History,
    junk: &mut JunkYard,
    mode: &Mode,
    buffers: Vec<&mut Buffer>,
    path: &Path,
) -> Vec<Action> {
    let mut actions = Vec::new();
    for buffer in buffers {
        let buffer = match buffer {
            Buffer::FileTree(it) => it,
            _ => continue,
        };

        if path.starts_with(junk.path.clone()) {
            remove_from_junkyard(junk, path);
        }

        let current_selection = match buffer.parent_cursor.as_ref() {
            Some(it) => get_selected_content_from_buffer(it, &buffer.current.buffer),
            None => None,
        };

        if let Some(parent) = path.parent() {
            if buffer.current.path.as_path() == parent {
                if let Some(cursor) = buffer.parent_cursor.as_mut() {
                    remove_line_from_buffer(cursor, mode, &mut buffer.current.buffer, path);
                }
            }

            if let (Some(cursor), FileTreeBufferSectionBuffer::Text(dir_path, text_buffer)) =
                (buffer.parent_cursor.as_mut(), &mut buffer.parent)
            {
                if dir_path.as_path() == parent {
                    remove_line_from_buffer(cursor, mode, text_buffer, path);
                }
            }

            if let (Some(cursor), FileTreeBufferSectionBuffer::Text(dir_path, text_buffer)) =
                (buffer.preview_cursor.as_mut(), &mut buffer.preview)
            {
                if dir_path.as_path() == parent {
                    remove_line_from_buffer(cursor, mode, text_buffer, path);
                }
            }
        }

        if let Some(selection) = current_selection {
            if let Some(cursor) = buffer.parent_cursor.as_mut() {
                let message = BufferMessage::SetCursorToLineContent(selection);
                yeet_buffer::update(
                    None,
                    Some(cursor),
                    mode,
                    &mut buffer.current.buffer,
                    std::slice::from_ref(&message),
                );
            }
        };

        if let Some(path) =
            selection::get_current_selected_path(buffer, buffer.parent_cursor.as_ref())
        {
            let selection = get_selection_from_history(history, &path).map(|s| s.to_owned());
            actions.push(Action::Load(
                FileTreeBufferSection::Preview,
                path,
                selection,
            ));
        }
    }

    actions
}
