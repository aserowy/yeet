use std::{mem, path::Path};

use yeet_buffer::model::{viewport::ViewPort, Cursor, Mode};

use crate::{
    action::Action,
    model::{self, history::History, mark::Marks, App, Buffer},
    update::{app, cursor, preview, selection},
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
    match path.parent() {
        Some(parent) => parent,
        None => &path,
    };

    navigate_to_path_with_selection(history, app, path.as_path(), &selection)
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

    if !path.exists() {}

    let mut actions = Vec::new();

    let current_selection = match selection {
        Some(it) => Some(it.to_owned()),
        None => {
            tracing::trace!("getting selection from history for path: {:?}", path);
            history::get_selection_from_history(history, path).map(|history| history.to_owned())
        }
    };

    tracing::trace!("resolved selection: {:?}", current_selection);

    let (current_id, load) =
        app::resolve_directory_buffer(&mut app.contents, path, &current_selection);
    actions.extend(load);

    let parent_id = if let (Some(parent), selection) = (path.parent(), path.file_name()) {
        let selection = selection.map(|selection| selection.to_string_lossy().to_string());
        let (id, load) = app::resolve_directory_buffer(&mut app.contents, parent, &selection);
        actions.extend(load);
        id
    } else {
        app::get_empty_buffer(&mut app.contents)
    };

    let preview_id = match &current_selection {
        Some(selected_history) => {
            let mut preview_path = path.to_path_buf();
            preview_path.push(selected_history);

            let selection = history::get_selection_from_history(history, preview_path.as_path())
                .map(|s| s.to_string());

            let (id, load) =
                app::resolve_directory_buffer(&mut app.contents, &preview_path, &selection);
            actions.extend(load);
            id
        }
        None => app::get_empty_buffer(&mut app.contents),
    };

    let (parent_vp, current_vp, preview_vp) = app::directory_viewports_mut(&mut app.window);
    parent_vp.buffer_id = parent_id;
    parent_vp.cursor = Cursor::default();
    current_vp.buffer_id = current_id;
    current_vp.cursor = Cursor::default();

    cursor::set_cursor_index(
        &mut app.contents,
        history,
        current_vp,
        &Mode::Normal,
        current_selection.as_deref(),
    );

    let parent_selection = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string());
    cursor::set_cursor_index(
        &mut app.contents,
        history,
        parent_vp,
        &Mode::Normal,
        parent_selection.as_deref(),
    );

    preview_vp.cursor = Cursor::default();
    preview::set_buffer_id(&mut app.contents, &mut app.window, preview_id);

    tracing::debug!(
        "navigate_to_path_with_selection returning {} actions",
        actions.len()
    );

    actions
}

#[tracing::instrument(skip(app))]
pub fn parent(app: &mut App) -> Vec<Action> {
    let (_, current_id, _) = app::directory_buffer_ids(app);
    let current_path = match app.contents.buffers.get(&current_id) {
        Some(Buffer::Directory(it)) => it.path.clone(),
        _ => return Vec::new(),
    };

    if let Some(path) = current_path.parent() {
        if current_path == path {
            return Vec::new();
        }

        let mut actions = Vec::new();

        let selection = path
            .file_name()
            .map(|oss| oss.to_string_lossy().to_string());

        let parent_id = if let Some(parent) = path.parent() {
            let (id, load) = app::resolve_directory_buffer(&mut app.contents, parent, &selection);
            actions.extend(load);
            id
        } else {
            app::get_empty_buffer(&mut app.contents)
        };

        let (parent_vp, current_vp, preview_vp) = app::directory_viewports_mut(&mut app.window);
        swap_viewport(parent_vp, preview_vp);
        swap_viewport(current_vp, preview_vp);

        parent_vp.buffer_id = parent_id;
        let directory = match app.contents.buffers.get_mut(&parent_vp.buffer_id) {
            Some(Buffer::Directory(it)) => it,
            _ => return actions,
        };

        if let Some(selection) = selection {
            cursor::set_cursor_index_to_selection(parent_vp, &Mode::Normal, directory, &selection);
        }

        actions
    } else {
        Vec::new()
    }
}

#[tracing::instrument(skip(app, history))]
pub fn selected(app: &mut App, history: &mut History) -> Vec<Action> {
    let (parent_vp, current_vp, preview_vp) = app::directory_viewports_mut(&mut app.window);
    let preview_buffer = match app.contents.buffers.get(&preview_vp.buffer_id) {
        Some(Buffer::Directory(it)) => it,
        _ => return Vec::new(),
    };

    history::add_history_entry(history, preview_buffer.path.as_path());

    swap_viewport(parent_vp, preview_vp);
    swap_viewport(current_vp, parent_vp);

    let actions = selection::refresh_preview_from_current_selection(app, history, None);

    let (_, _, preview_vp) = app::directory_viewports_mut(&mut app.window);
    cursor::set_cursor_index(&mut app.contents, history, preview_vp, &Mode::Normal, None);

    actions
}

fn swap_viewport(vp1: &mut ViewPort, vp2: &mut ViewPort) {
    mem::swap(&mut vp1.buffer_id, &mut vp2.buffer_id);
    mem::swap(&mut vp1.cursor, &mut vp2.cursor);
    mem::swap(&mut vp1.horizontal_index, &mut vp2.horizontal_index);
    mem::swap(&mut vp1.vertical_index, &mut vp2.vertical_index);
}
