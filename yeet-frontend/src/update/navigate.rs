use std::{mem, path::Path};

use yeet_buffer::{
    message::ViewPortDirection,
    model::{Cursor, TextBuffer},
};

use crate::{
    action::Action,
    model::{history::History, mark::Marks, App, Buffer, DirectoryPane},
    update::app,
};

use super::{history, selection};

use std::path::PathBuf;

#[tracing::instrument(skip(app))]
pub fn mark(app: &mut App, history: &History, marks: &Marks, char: &char) -> Vec<Action> {
    let (_, current_id, _) = app::directory_buffer_ids(app);
    let _buffer = match app.buffers.get(&current_id) {
        Some(Buffer::Directory(it)) => it,
        _ => return Vec::new(),
    };

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
    let (_, current_id, _) = app::directory_buffer_ids(app);
    let _buffer = match app.buffers.get(&current_id) {
        Some(Buffer::Directory(it)) => it,
        _ => return Vec::new(),
    };

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
    let (_, current_id, _) = app::directory_buffer_ids(app);
    let _buffer = match app.buffers.get(&current_id) {
        Some(Buffer::Directory(it)) => it,
        _ => return Vec::new(),
    };

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

    let (_, _, preview_id) = app::directory_buffer_ids(app);
    if let Some(Buffer::Directory(buffer)) = app.buffers.get_mut(&preview_id) {
        buffer.buffer = TextBuffer::default();
        buffer.path = PathBuf::default();
    }

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
    let (parent_id, current_id, preview_id) = app::directory_buffer_ids(app);
    let current_path = match app.buffers.get(&current_id) {
        Some(Buffer::Directory(it)) => it.path.clone(),
        _ => return Vec::new(),
    };

    if let Some(path) = current_path.parent() {
        if current_path == path {
            return Vec::new();
        }

        let mut actions = Vec::new();

        if let Some(parent) = path.parent() {
            tracing::trace!("loading parent: {:?}", parent);

            actions.push(Action::Load(
                DirectoryPane::Parent,
                parent.to_path_buf(),
                path.file_name().map(|it| it.to_string_lossy().to_string()),
            ));
        }

        let parent_buffer = match app.buffers.remove(&parent_id) {
            Some(Buffer::Directory(it)) => it,
            _ => return Vec::new(),
        };
        let current_buffer = match app.buffers.remove(&current_id) {
            Some(Buffer::Directory(it)) => it,
            _ => return Vec::new(),
        };
        let preview_buffer = match app.buffers.remove(&preview_id) {
            Some(Buffer::Directory(it)) => it,
            _ => return Vec::new(),
        };

        let (mut parent_buffer, mut current_buffer, mut preview_buffer) =
            (parent_buffer, current_buffer, preview_buffer);

        preview_buffer.path = mem::replace(&mut current_buffer.path, path.to_path_buf());
        preview_buffer.buffer = mem::take(&mut current_buffer.buffer);
        mem::swap(&mut preview_buffer.buffer, &mut parent_buffer.buffer);

        mem_swap_cursor(
            &mut current_buffer.buffer.cursor,
            &mut parent_buffer.buffer.cursor,
        );

        let mut vp = {
            let (_, current, _) = app::directory_viewports(app);
            current.clone()
        };

        yeet_buffer::update_viewport_by_direction(
            &mut vp,
            &mut current_buffer.buffer,
            &ViewPortDirection::CenterOnCursor,
        );

        app.buffers
            .insert(parent_id, Buffer::Directory(parent_buffer));
        app.buffers
            .insert(current_id, Buffer::Directory(current_buffer));
        app.buffers
            .insert(preview_id, Buffer::Directory(preview_buffer));

        actions
    } else {
        Vec::new()
    }
}

#[tracing::instrument(skip(app, history))]
pub fn selected(app: &mut App, history: &mut History) -> Vec<Action> {
    let (parent_id, current_id, preview_id) = app::directory_buffer_ids(app);
    let parent_buffer = match app.buffers.remove(&parent_id) {
        Some(Buffer::Directory(it)) => it,
        _ => return Vec::new(),
    };
    let current_buffer = match app.buffers.remove(&current_id) {
        Some(Buffer::Directory(it)) => it,
        _ => return Vec::new(),
    };
    let preview_buffer = match app.buffers.remove(&preview_id) {
        Some(Buffer::Directory(it)) => it,
        _ => return Vec::new(),
    };

    let (mut parent_buffer, mut current_buffer, preview_buffer) =
        (parent_buffer, current_buffer, preview_buffer);

    if let Some(selected) =
        selection::get_current_selected_path(&current_buffer, Some(&current_buffer.buffer.cursor))
    {
        if current_buffer.path == selected || !selected.is_dir() {
            return Vec::new();
        }

        history::add_history_entry(history, selected.as_path());

        parent_buffer.path = mem::replace(&mut current_buffer.path, selected.to_path_buf());
        parent_buffer.buffer = mem::take(&mut current_buffer.buffer);
        mem::swap(&mut parent_buffer.buffer, &mut current_buffer.buffer);

        let mut actions = Vec::new();
        let history =
            history::get_selection_from_history(history, selected.as_path()).map(|s| s.to_string());

        actions.push(Action::Load(
            DirectoryPane::Current,
            selected.to_path_buf(),
            history,
        ));

        let mut vp = {
            let (_, current, _) = app::directory_viewports(app);
            current.clone()
        };
        yeet_buffer::update_viewport_by_direction(
            &mut vp,
            &mut current_buffer.buffer,
            &ViewPortDirection::CenterOnCursor,
        );

        app.buffers
            .insert(parent_id, Buffer::Directory(parent_buffer));
        app.buffers
            .insert(current_id, Buffer::Directory(current_buffer));
        app.buffers
            .insert(preview_id, Buffer::Directory(preview_buffer));

        actions
    } else {
        app.buffers
            .insert(parent_id, Buffer::Directory(parent_buffer));
        app.buffers
            .insert(current_id, Buffer::Directory(current_buffer));
        app.buffers
            .insert(preview_id, Buffer::Directory(preview_buffer));
        Vec::new()
    }
}

fn mem_swap_cursor(dest_cursor: &mut Cursor, src_cursor: &mut Cursor) {
    mem::swap(
        &mut dest_cursor.vertical_index,
        &mut src_cursor.vertical_index,
    );
    mem::swap(
        &mut dest_cursor.horizontal_index,
        &mut src_cursor.horizontal_index,
    );
}
