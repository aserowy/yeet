use std::{mem, path::Path};

use yeet_buffer::model::{viewport::ViewPort, Cursor, CursorPosition, TextBuffer};

use crate::{
    action::Action,
    model::{FileTreeBufferSection, FileTreeBufferSectionBuffer, Model},
};

use super::{history, selection};

#[tracing::instrument(skip(model))]
pub fn navigate_to_mark(char: &char, model: &mut Model) -> Vec<Action> {
    let path = match model.marks.entries.get(char) {
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

    navigate_to_path_with_selection(model, path, &selection)
}

#[tracing::instrument(skip(model))]
pub fn navigate_to_path(model: &mut Model, path: &Path) -> Vec<Action> {
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

    navigate_to_path_with_selection(model, path, &selection)
}

pub fn navigate_to_path_as_preview(model: &mut Model, path: &Path) -> Vec<Action> {
    let selection = path
        .file_name()
        .map(|oss| oss.to_string_lossy().to_string());

    let path = match path.parent() {
        Some(parent) => parent,
        None => path,
    };

    navigate_to_path_with_selection(model, path, &selection)
}

#[tracing::instrument(skip(model))]
pub fn navigate_to_path_with_selection(
    model: &mut Model,
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
            history::get_selection_from_history(&model.history, path)
                .map(|history| history.to_owned())
        }
    };

    tracing::trace!("resolved selection: {:?}", selection);

    model.files.preview = FileTreeBufferSectionBuffer::None;

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

#[tracing::instrument(skip(model))]
pub fn navigate_to_parent(model: &mut Model) -> Vec<Action> {
    if let Some(path) = model.files.current.path.clone().parent() {
        if model.files.current.path == path {
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
            match mem::replace(&mut model.files.parent, FileTreeBufferSectionBuffer::None) {
                FileTreeBufferSectionBuffer::Text(_, buffer) => buffer,
                FileTreeBufferSectionBuffer::Image(_, _) | FileTreeBufferSectionBuffer::None => {
                    TextBuffer::default()
                }
            };

        let current_path = mem::replace(&mut model.files.current.path, path.to_path_buf());
        let current_buffer = mem::replace(&mut model.files.current.buffer, parent_buffer);

        model.files.preview = FileTreeBufferSectionBuffer::Text(current_path, current_buffer);
        model.files.preview_cursor = Some(Default::default());

        mem_swap_viewport(&mut model.files.current_vp, &mut model.files.parent_vp);
        mem_swap_viewport(&mut model.files.parent_vp, &mut model.files.preview_vp);

        mem_swap_cursor(
            &mut model.files.current_cursor,
            &mut model.files.parent_cursor,
        );
        mem_swap_cursor(
            &mut model.files.parent_cursor,
            &mut model.files.preview_cursor,
        );

        actions
    } else {
        Vec::new()
    }
}

#[tracing::instrument(skip(model))]
pub fn navigate_to_selected(model: &mut Model) -> Vec<Action> {
    if let Some(selected) = selection::get_current_selected_path(model) {
        if model.files.current.path == selected || !selected.is_dir() {
            return Vec::new();
        }

        history::add_history_entry(&mut model.history, selected.as_path());

        let mut actions = Vec::new();
        let preview_buffer =
            match mem::replace(&mut model.files.preview, FileTreeBufferSectionBuffer::None) {
                FileTreeBufferSectionBuffer::Text(_, buffer) => buffer,
                FileTreeBufferSectionBuffer::Image(_, _) | FileTreeBufferSectionBuffer::None => {
                    let history =
                        history::get_selection_from_history(&model.history, selected.as_path())
                            .map(|s| s.to_string());

                    actions.push(Action::Load(
                        FileTreeBufferSection::Current,
                        selected.to_path_buf(),
                        history,
                    ));

                    TextBuffer::default()
                }
            };

        model.files.parent = FileTreeBufferSectionBuffer::Text(
            mem::replace(&mut model.files.current.path, selected.to_path_buf()),
            mem::replace(&mut model.files.current.buffer, preview_buffer),
        );

        mem_swap_cursor(
            &mut model.files.current_cursor,
            &mut model.files.parent_cursor,
        );
        mem_swap_cursor(
            &mut model.files.current_cursor,
            &mut model.files.preview_cursor,
        );

        if let Some(cursor) = &mut model.files.current_cursor {
            cursor.horizontal_index = CursorPosition::Absolute {
                current: 0,
                expanded: 0,
            };
        } else {
            model.files.current_cursor = Some(Default::default());
        }

        if let Some(selected) = selection::get_current_selected_path(model) {
            tracing::trace!("loading selection: {:?}", selected);

            let history = history::get_selection_from_history(&model.history, selected.as_path())
                .map(|s| s.to_string());

            actions.push(Action::Load(
                FileTreeBufferSection::Preview,
                selected.to_path_buf(),
                history,
            ));
        }

        mem_swap_viewport(&mut model.files.current_vp, &mut model.files.parent_vp);
        mem_swap_viewport(&mut model.files.current_vp, &mut model.files.preview_vp);

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
