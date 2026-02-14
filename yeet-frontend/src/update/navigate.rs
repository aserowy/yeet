use std::path::Path;

use crate::{
    action::Action,
    model::{history::History, mark::Marks, App, Buffer, DirectoryBuffer, DirectoryPane},
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

    let selection = match selection {
        Some(it) => Some(it.to_owned()),
        None => {
            tracing::trace!("getting selection from history for path: {:?}", path);
            history::get_selection_from_history(history, path).map(|history| history.to_owned())
        }
    };

    tracing::trace!("resolved selection: {:?}", selection);

    let current_id = get_or_create_directory_buffer_id(app, path);
    let parent_id = path
        .parent()
        .filter(|parent| *parent != path)
        .map(|parent| get_or_create_directory_buffer_id(app, parent))
        .unwrap_or_else(|| get_or_create_empty_directory_buffer_id(app));
    let preview_id = selection
        .as_ref()
        .map(|selection| path.join(selection))
        .and_then(|preview_path| {
            if preview_path.exists() {
                Some(get_or_create_directory_buffer_id(
                    app,
                    preview_path.as_path(),
                ))
            } else {
                None
            }
        })
        .unwrap_or_else(|| get_or_create_empty_directory_buffer_id(app));

    let (parent_vp, current_vp, preview_vp) = app::directory_viewports_mut(app);
    parent_vp.buffer_id = parent_id;
    current_vp.buffer_id = current_id;
    preview_vp.buffer_id = preview_id;

    let mut actions = Vec::new();
    actions.push(Action::Load(
        DirectoryPane::Current,
        path.to_path_buf(),
        selection.clone(),
    ));

    let parent = path.parent();
    if let Some(parent) = parent {
        if parent != path {
            actions.push(Action::Load(
                DirectoryPane::Parent,
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

        let current_id = get_or_create_directory_buffer_id(app, path);
        let parent_id = path
            .parent()
            .filter(|parent| *parent != path)
            .map(|parent| get_or_create_directory_buffer_id(app, parent))
            .unwrap_or_else(|| get_or_create_empty_directory_buffer_id(app));
        let preview_id = get_or_create_empty_directory_buffer_id(app);

        let (parent_vp, current_vp, preview_vp) = app::directory_viewports_mut(app);
        parent_vp.buffer_id = parent_id;
        current_vp.buffer_id = current_id;
        preview_vp.buffer_id = preview_id;

        if let Some(parent) = path.parent() {
            tracing::trace!("loading parent: {:?}", parent);

            actions.push(Action::Load(
                DirectoryPane::Parent,
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
    let (_, current_id, _) = app::directory_buffer_ids(app);
    let current_buffer = match app.buffers.get(&current_id) {
        Some(Buffer::Directory(it)) => it,
        _ => return Vec::new(),
    };

    if let Some(selected) =
        selection::get_current_selected_path(current_buffer, Some(&current_buffer.buffer.cursor))
    {
        if current_buffer.path == selected || !selected.is_dir() {
            return Vec::new();
        }

        history::add_history_entry(history, selected.as_path());

        let current_id = get_or_create_directory_buffer_id(app, selected.as_path());
        let parent_id = selected
            .parent()
            .filter(|parent| *parent != selected.as_path())
            .map(|parent| get_or_create_directory_buffer_id(app, parent))
            .unwrap_or_else(|| get_or_create_empty_directory_buffer_id(app));
        let preview_id = get_or_create_empty_directory_buffer_id(app);

        let (parent_vp, current_vp, preview_vp) = app::directory_viewports_mut(app);
        parent_vp.buffer_id = parent_id;
        current_vp.buffer_id = current_id;
        preview_vp.buffer_id = preview_id;

        let mut actions = Vec::new();
        let history =
            history::get_selection_from_history(history, selected.as_path()).map(|s| s.to_string());

        actions.push(Action::Load(
            DirectoryPane::Current,
            selected.to_path_buf(),
            history,
        ));

        actions
    } else {
        Vec::new()
    }
}

fn get_or_create_directory_buffer_id(app: &mut App, path: &Path) -> usize {
    if let Some((id, _)) = app.buffers.iter().find(|(_, buffer)| match buffer {
        Buffer::Directory(it) => it.path.as_path() == path,
        _ => false,
    }) {
        return *id;
    }

    let id = app::get_next_buffer_id(app);
    app.buffers.insert(
        id,
        Buffer::Directory(DirectoryBuffer {
            path: path.to_path_buf(),
            ..Default::default()
        }),
    );
    id
}

fn get_or_create_empty_directory_buffer_id(app: &mut App) -> usize {
    if let Some((id, _)) = app.buffers.iter().find(|(_, buffer)| match buffer {
        Buffer::Directory(it) => it.path.as_os_str().is_empty(),
        _ => false,
    }) {
        return *id;
    }

    let id = app::get_next_buffer_id(app);
    app.buffers
        .insert(id, Buffer::Directory(DirectoryBuffer::default()));
    id
}
