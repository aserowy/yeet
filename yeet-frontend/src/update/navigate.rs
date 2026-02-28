use std::path::Path;

use yeet_buffer::model::{Cursor, Mode};

use crate::{
    action::Action,
    model::{self, history::History, mark::Marks, App, Buffer},
    update::{app, cursor, preview},
};

use super::history;

#[tracing::instrument(skip(app))]
pub fn mark(app: &mut App, history: &History, marks: &Marks, char: &char) -> Vec<Action> {
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

    navigate_to_path_with_selection(history, app, path, &selection)
}

#[tracing::instrument(skip(app, history))]
pub fn path(app: &mut App, history: &History, path: &Path) -> Vec<Action> {
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

    navigate_to_path_with_selection(history, app, path, &selection)
}

pub fn path_as_preview(app: &mut App, history: &History, path: &Path) -> Vec<Action> {
    let selection = path
        .file_name()
        .map(|oss| oss.to_string_lossy().to_string());

    let path = match path.parent() {
        Some(parent) => parent,
        None => path,
    };

    navigate_to_path_with_selection(history, app, path, &selection)
}

#[tracing::instrument(skip(app, history))]
pub fn navigate_to_path_with_selection(
    history: &History,
    app: &mut App,
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

    let mut actions = Vec::new();

    let selection = match selection {
        Some(it) => Some(it.to_owned()),
        None => {
            tracing::trace!("getting selection from history for path: {:?}", path);
            history::get_selection_from_history(history, path).map(|history| history.to_owned())
        }
    };

    tracing::trace!("resolved selection: {:?}", selection);

    let (current_id, load) = app::get_or_create_directory_buffer(app, path, &selection);
    actions.extend(load);

    let parent_id = if let (Some(parent), selection) = (path.parent(), path.file_name()) {
        let selection = selection.map(|selection| selection.to_string_lossy().to_string());
        let (id, load) = app::get_or_create_directory_buffer(app, parent, &selection);
        actions.extend(load);
        id
    } else {
        app::create_empty_buffer(app)
    };

    let preview_id = match &selection {
        Some(selected_history) => {
            let mut preview_path = path.to_path_buf();
            preview_path.push(selected_history);

            let selection = history::get_selection_from_history(history, preview_path.as_path())
                .map(|s| s.to_string());

            let (id, load) = app::get_or_create_directory_buffer(app, &preview_path, &selection);
            actions.extend(load);
            id
        }
        None => app::create_empty_buffer(app),
    };

    let (parent_vp, current_vp, _) = app::directory_viewports_mut(app);
    parent_vp.buffer_id = parent_id;
    parent_vp.cursor = Cursor::default();
    current_vp.buffer_id = current_id;
    current_vp.cursor = Cursor::default();

    preview::set_buffer_id(app, preview_id);
    let (_, _, preview_vp) = app::directory_viewports_mut(app);
    preview_vp.cursor = Cursor::default();

    // Set cursors to selection for already-loaded buffers.
    // For buffers pending load, enumeration::finish will handle cursor positioning.
    set_cursor_for_existing_buffer(app, current_id, &selection, history, path);

    let parent_selection = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string());
    if let Some(parent) = path.parent() {
        set_cursor_for_existing_buffer(app, parent_id, &parent_selection, history, parent);
    }

    tracing::debug!(
        "navigate_to_path_with_selection returning {} actions",
        actions.len()
    );
    actions
}

#[tracing::instrument(skip(app))]
pub fn parent(app: &mut App) -> Vec<Action> {
    let (_, current_id, _) = app::directory_buffer_ids(app);
    let current_path = match app.buffers.get(&current_id) {
        Some(Buffer::Directory(it)) => it.path.clone(),
        _ => return Vec::new(),
    };

    if let Some(path) = current_path.parent() {
        if current_path == path {
            return Vec::new();
        }

        let mut actions = Vec::new();

        let parent_id = if let (Some(parent), selection) = (path.parent(), path.file_name()) {
            let selection = selection.map(|selection| selection.to_string_lossy().to_string());
            let (id, load) = app::get_or_create_directory_buffer(app, parent, &selection);
            actions.extend(load);
            id
        } else {
            app::create_empty_buffer(app)
        };

        let (parent_vp, current_vp, _) = app::directory_viewports_mut(app);
        let preview_id = current_vp.buffer_id;
        let old_current_cursor = current_vp.cursor.clone();
        let old_parent_cursor = parent_vp.cursor.clone();

        current_vp.buffer_id = parent_vp.buffer_id;
        current_vp.cursor = old_parent_cursor;

        parent_vp.buffer_id = parent_id;
        parent_vp.cursor = Cursor::default();

        preview::set_buffer_id(app, preview_id);
        let (_, _, preview_vp) = app::directory_viewports_mut(app);
        preview_vp.cursor = old_current_cursor;

        actions
    } else {
        Vec::new()
    }
}

#[tracing::instrument(skip(app, history))]
pub fn selected(app: &mut App, history: &mut History) -> Vec<Action> {
    let (_, _, preview_id) = app::directory_buffer_ids(app);
    let preview_vp = app::get_viewport_by_buffer_id(app, preview_id);
    let preview_cursor = preview_vp.map(|vp| vp.cursor.clone());
    let preview_buffer = match app.buffers.get(&preview_id) {
        Some(Buffer::Directory(it)) => it,
        _ => return Vec::new(),
    };

    history::add_history_entry(history, preview_buffer.path.as_path());

    let current_preview_selection = preview_cursor
        .as_ref()
        .and_then(|cursor| model::get_selected_path(preview_buffer, cursor));

    let mut actions = Vec::new();
    let preview_id = match &current_preview_selection {
        Some(preview_path) => {
            let selection = history::get_selection_from_history(history, preview_path.as_path())
                .map(|s| s.to_string());

            let (id, load) = app::get_or_create_directory_buffer(app, preview_path, &selection);
            actions.extend(load);
            id
        }
        None => {
            if !preview_buffer.path.is_dir() {
                tracing::warn!("no selection in current buffer, cannot navigate to selected");
                return Vec::new();
            }
            app::create_empty_buffer(app)
        }
    };

    let (parent_vp, current_vp, preview_vp) = app::directory_viewports_mut(app);
    let old_current_cursor = current_vp.cursor.clone();
    let old_preview_cursor = preview_vp.cursor.clone();

    parent_vp.buffer_id = current_vp.buffer_id;
    parent_vp.cursor = old_current_cursor;

    current_vp.buffer_id = preview_vp.buffer_id;
    current_vp.cursor = old_preview_cursor;

    preview::set_buffer_id(app, preview_id);
    let (_, _, preview_vp) = app::directory_viewports_mut(app);
    preview_vp.cursor = Cursor::default();

    actions
}

/// Sets the cursor on the viewport for `buffer_id` to the given selection (or history fallback).
/// This is a no-op if the buffer is not yet loaded (empty lines) or has no matching viewport.
fn set_cursor_for_existing_buffer(
    app: &mut App,
    buffer_id: usize,
    selection: &Option<String>,
    history: &History,
    path: &Path,
) {
    let App {
        buffers, window, ..
    } = app;
    let mut viewport = app::get_viewport_by_buffer_id_mut(window, buffer_id);
    let buffer = match buffers.get_mut(&buffer_id) {
        Some(Buffer::Directory(it)) => it,
        _ => return,
    };

    if buffer.buffer.lines.is_empty() {
        return;
    }

    if let Some(selection) = selection {
        if cursor::set_cursor_index_to_selection(
            viewport.as_deref_mut(),
            &Mode::Navigation,
            &mut buffer.buffer,
            selection,
        ) {
            return;
        }
    }

    cursor::set_cursor_index_with_history(
        history,
        viewport,
        &Mode::Navigation,
        &mut buffer.buffer,
        path,
    );
}
