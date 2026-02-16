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

    let current_id = app::get_or_create_directory_buffer_with_id(app, path);

    let parent_id = match path.parent() {
        Some(parent) => {
            actions.push(Action::Load(path.to_path_buf(), selection.clone()));
            app::get_or_create_directory_buffer_with_id(app, parent)
        }
        None => app::create_empty_buffer_with_id(app),
    };

    let preview_id = match &selection {
        Some(selected_history) => {
            let mut preview_path = path.to_path_buf();
            preview_path.push(selected_history);

            actions.push(Action::Load(
                preview_path.to_path_buf(),
                history::get_selection_from_history(history, preview_path.as_path())
                    .map(|s| s.to_string()),
            ));

            app::get_or_create_directory_buffer_with_id(app, &preview_path)
        }
        None => app::create_empty_buffer_with_id(app),
    };

    let (parent_vp, current_vp, preview_vp) = app::directory_viewports_mut(app);
    parent_vp.buffer_id = parent_id;
    current_vp.buffer_id = current_id;
    preview_vp.buffer_id = preview_id;

    actions.push(Action::Load(path.to_path_buf(), selection.clone()));

    let parent = path.parent();
    if let Some(parent) = parent {
        if parent != path {
            actions.push(Action::Load(
                parent.to_path_buf(),
                path.file_name().map(|it| it.to_string_lossy().to_string()),
            ));
        }
    }

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

        let parent_id = path
            .parent()
            .filter(|parent| *parent != path)
            .map(|parent| app::get_or_create_directory_buffer_with_id(app, parent))
            .unwrap_or_else(|| app::create_empty_buffer_with_id(app));

        let (parent_vp, current_vp, preview_vp) = app::directory_viewports_mut(app);
        preview_vp.buffer_id = current_vp.buffer_id;
        current_vp.buffer_id = parent_vp.buffer_id;
        parent_vp.buffer_id = parent_id;

        if let Some(parent) = path.parent() {
            tracing::trace!("loading parent: {:?}", parent);

            actions.push(Action::Load(
                parent.to_path_buf(),
                path.file_name().map(|it| it.to_string_lossy().to_string()),
            ));
        }

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

    let current_preview_selection = match selection::get_current_selected_path(
        preview_buffer,
        Some(&preview_buffer.buffer.cursor),
    ) {
        Some(selection) => selection,
        None => {
            tracing::warn!("no selection in current buffer, cannot navigate to selected");
            return Vec::new();
        }
    };

    let preview_id = app::get_or_create_directory_buffer_with_id(app, &current_preview_selection);
    let (parent_vp, current_vp, preview_vp) = app::directory_viewports_mut(app);

    parent_vp.buffer_id = current_vp.buffer_id;
    current_vp.buffer_id = preview_vp.buffer_id;
    preview_vp.buffer_id = preview_id;

    vec![Action::Load(
        current_preview_selection.to_path_buf(),
        history::get_selection_from_history(history, current_preview_selection.as_path())
            .map(|s| s.to_string()),
    )]
}
