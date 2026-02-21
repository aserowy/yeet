use std::path::Path;

use crate::{
    action::Action,
    model::{history::History, mark::Marks, App, Buffer},
    update::app,
};

use super::{history, selection};

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

    let (parent_vp, current_vp, preview_vp) = app::directory_viewports_mut(app);
    parent_vp.buffer_id = parent_id;
    current_vp.buffer_id = current_id;
    preview_vp.buffer_id = preview_id;

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

        let (parent_vp, current_vp, preview_vp) = app::directory_viewports_mut(app);
        preview_vp.buffer_id = current_vp.buffer_id;
        current_vp.buffer_id = parent_vp.buffer_id;
        parent_vp.buffer_id = parent_id;

        actions
    } else {
        Vec::new()
    }
}

#[tracing::instrument(skip(app, history))]
pub fn selected(app: &mut App, history: &mut History) -> Vec<Action> {
    let (_, _, preview_id) = app::directory_buffer_ids(app);
    let preview_buffer = match app.buffers.get(&preview_id) {
        Some(Buffer::Directory(it)) => it,
        _ => return Vec::new(),
    };

    history::add_history_entry(history, preview_buffer.path.as_path());

    let current_preview_selection =
        selection::get_current_selected_path(preview_buffer, Some(&preview_buffer.buffer.cursor));

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

    parent_vp.buffer_id = current_vp.buffer_id;
    current_vp.buffer_id = preview_vp.buffer_id;
    preview_vp.buffer_id = preview_id;

    actions
}
