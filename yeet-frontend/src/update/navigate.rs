use std::{mem, path::Path};

use yeet_buffer::model::{viewport::ViewPort, Cursor, CursorPosition, TextBuffer};

use crate::{
    action::Action,
    model::{
        history::History, mark::Marks, FileTreeBuffer, FileTreeBufferSection,
        FileTreeBufferSectionBuffer,
    },
};

use super::{history, selection};

#[tracing::instrument(skip(buffer))]
pub fn mark(
    history: &History,
    marks: &Marks,
    buffer: &mut FileTreeBuffer,
    char: &char,
) -> Vec<Action> {
    let path = match marks.entries.get(char) {
        Some(it) => it.clone(),
        None => return Vec::new(),
    };

    let selection = path
        .file_name()
        .map(|oss| oss.to_string_lossy().to_string());

    let path = match path.parent() {
        Some(parent) => parent,
        None => &path,
    };

    navigate_to_path_with_selection(history, buffer, path, &selection)
}

#[tracing::instrument(skip(buffer, history))]
pub fn path(history: &History, buffer: &mut FileTreeBuffer, path: &Path) -> Vec<Action> {
    let (path, selection) = if path.is_file() {
        tracing::info!("path is a file, not a directory: {:?}", path);

        let selection = path
            .file_name()
            .map(|oss| oss.to_string_lossy().to_string());

        match path.parent() {
            Some(parent) => (parent, selection),
            None => {
                tracing::warn!(
                    "parent from path with file name could not get resolved: {:?}",
                    path
                );
                return Vec::new();
            }
        }
    } else {
        (path, None)
    };

    navigate_to_path_with_selection(history, buffer, path, &selection)
}

pub fn path_as_preview(history: &History, buffer: &mut FileTreeBuffer, path: &Path) -> Vec<Action> {
    let selection = path
        .file_name()
        .map(|oss| oss.to_string_lossy().to_string());

    let path = match path.parent() {
        Some(parent) => parent,
        None => path,
    };

    navigate_to_path_with_selection(history, buffer, path, &selection)
}

#[tracing::instrument(skip(buffer, history))]
pub fn navigate_to_path_with_selection(
    history: &History,
    buffer: &mut FileTreeBuffer,
    path: &Path,
    selection: &Option<String>,
) -> Vec<Action> {
    if path.is_file() {
        tracing::warn!("path is a file, not a directory: {:?}", path);
        return Vec::new();
    }

    if !path.exists() {
        tracing::warn!("path does not exist: {:?}", path);
        return Vec::new();
    }

    let selection = match selection {
        Some(it) => Some(it.to_owned()),
        None => {
            tracing::trace!("getting selection from history for path: {:?}", path);
            history::get_selection_from_history(history, path).map(|history| history.to_owned())
        }
    };

    tracing::trace!("resolved selection: {:?}", selection);

    buffer.preview = FileTreeBufferSectionBuffer::None;

    let mut actions = Vec::new();
    actions.push(Action::Load(
        FileTreeBufferSection::Current,
        path.to_path_buf(),
        selection.clone(),
    ));

    let parent = path.parent();
    if let Some(parent) = parent {
        if parent != path {
            actions.push(Action::Load(
                FileTreeBufferSection::Parent,
                parent.to_path_buf(),
                path.file_name().map(|it| it.to_string_lossy().to_string()),
            ));
        }
    }

    actions
}

#[tracing::instrument(skip(buffer))]
pub fn parent(buffer: &mut FileTreeBuffer) -> Vec<Action> {
    if let Some(path) = buffer.current.path.clone().parent() {
        if buffer.current.path == path {
            return Vec::new();
        }

        let mut actions = Vec::new();

        if let Some(parent) = path.parent() {
            tracing::trace!("loading parent: {:?}", parent);

            actions.push(Action::Load(
                FileTreeBufferSection::Parent,
                parent.to_path_buf(),
                path.file_name().map(|it| it.to_string_lossy().to_string()),
            ));
        }

        let parent_buffer =
            match mem::replace(&mut buffer.parent, FileTreeBufferSectionBuffer::None) {
                FileTreeBufferSectionBuffer::Text(_, buffer) => buffer,
                FileTreeBufferSectionBuffer::Image(_, _) | FileTreeBufferSectionBuffer::None => {
                    TextBuffer::default()
                }
            };

        let current_path = mem::replace(&mut buffer.current.path, path.to_path_buf());
        let current_buffer = mem::replace(&mut buffer.current.buffer, parent_buffer);

        buffer.preview = FileTreeBufferSectionBuffer::Text(current_path, current_buffer);
        buffer.preview_cursor = Some(Default::default());

        mem_swap_viewport(&mut buffer.current_vp, &mut buffer.parent_vp);
        mem_swap_viewport(&mut buffer.parent_vp, &mut buffer.preview_vp);

        mem_swap_cursor(&mut buffer.current_cursor, &mut buffer.parent_cursor);
        mem_swap_cursor(&mut buffer.parent_cursor, &mut buffer.preview_cursor);

        actions
    } else {
        Vec::new()
    }
}

#[tracing::instrument(skip(buffer, history))]
pub fn selected(history: &mut History, buffer: &mut FileTreeBuffer) -> Vec<Action> {
    if let Some(selected) = selection::get_current_selected_path(buffer) {
        if buffer.current.path == selected || !selected.is_dir() {
            return Vec::new();
        }

        history::add_history_entry(history, selected.as_path());

        let mut actions = Vec::new();
        let preview_buffer =
            match mem::replace(&mut buffer.preview, FileTreeBufferSectionBuffer::None) {
                FileTreeBufferSectionBuffer::Text(_, buffer) => buffer,
                FileTreeBufferSectionBuffer::Image(_, _) | FileTreeBufferSectionBuffer::None => {
                    let history = history::get_selection_from_history(history, selected.as_path())
                        .map(|s| s.to_string());

                    actions.push(Action::Load(
                        FileTreeBufferSection::Current,
                        selected.to_path_buf(),
                        history,
                    ));

                    TextBuffer::default()
                }
            };

        buffer.parent = FileTreeBufferSectionBuffer::Text(
            mem::replace(&mut buffer.current.path, selected.to_path_buf()),
            mem::replace(&mut buffer.current.buffer, preview_buffer),
        );

        mem_swap_cursor(&mut buffer.current_cursor, &mut buffer.parent_cursor);
        mem_swap_cursor(&mut buffer.current_cursor, &mut buffer.preview_cursor);

        if let Some(cursor) = &mut buffer.current_cursor {
            cursor.horizontal_index = CursorPosition::Absolute {
                current: 0,
                expanded: 0,
            };
        } else {
            buffer.current_cursor = Some(Default::default());
        }

        if let Some(selected) = selection::get_current_selected_path(buffer) {
            tracing::trace!("loading selection: {:?}", selected);

            let history = history::get_selection_from_history(history, selected.as_path())
                .map(|s| s.to_string());

            actions.push(Action::Load(
                FileTreeBufferSection::Preview,
                selected.to_path_buf(),
                history,
            ));
        }

        mem_swap_viewport(&mut buffer.current_vp, &mut buffer.parent_vp);
        mem_swap_viewport(&mut buffer.current_vp, &mut buffer.preview_vp);

        actions
    } else {
        Vec::new()
    }
}

fn mem_swap_viewport(dest_viewport: &mut ViewPort, src_viewport: &mut ViewPort) {
    mem::swap(
        &mut dest_viewport.horizontal_index,
        &mut src_viewport.horizontal_index,
    );
    mem::swap(
        &mut dest_viewport.vertical_index,
        &mut src_viewport.vertical_index,
    );
}

fn mem_swap_cursor(dest_cursor: &mut Option<Cursor>, src_cursor: &mut Option<Cursor>) {
    if let (Some(dest_cursor), Some(src_cursor)) = (dest_cursor, src_cursor) {
        mem::swap(
            &mut dest_cursor.vertical_index,
            &mut src_cursor.vertical_index,
        );
    }
}
