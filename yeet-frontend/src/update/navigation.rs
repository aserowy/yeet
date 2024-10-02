use std::{mem, path::Path};

use yeet_buffer::{
    message::BufferMessage,
    model::{Buffer, Mode},
};

use crate::{
    action::Action,
    model::{BufferType, Model, WindowType},
};

use super::{cursor, history, selection};

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

    model.files.preview = BufferType::None;

    let mut actions = Vec::new();
    actions.push(Action::Load(
        WindowType::Current,
        path.to_path_buf(),
        selection.clone(),
    ));

    let parent = path.parent();
    if let Some(parent) = parent {
        if parent != path {
            actions.push(Action::Load(
                WindowType::Parent,
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
                WindowType::Parent,
                parent.to_path_buf(),
                path.file_name().map(|it| it.to_string_lossy().to_string()),
            ));
        }

        let parent_buffer = match mem::replace(&mut model.files.parent, BufferType::None) {
            BufferType::Text(_, buffer) => buffer,
            BufferType::Image(_, _) | BufferType::None => Buffer::default(),
        };

        let current_path = mem::replace(&mut model.files.current.path, path.to_path_buf());
        let current_buffer =
            mem_replace_buffer(&model.mode, &mut model.files.current.buffer, parent_buffer);

        model.files.preview = BufferType::Text(current_path, current_buffer);

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

        let preview_buffer = match mem::replace(&mut model.files.preview, BufferType::None) {
            BufferType::Text(_, buffer) => buffer,
            BufferType::Image(_, _) | BufferType::None => Buffer::default(),
        };

        model.files.parent = BufferType::Text(
            mem::replace(&mut model.files.current.path, selected.to_path_buf()),
            mem_replace_buffer(&model.mode, &mut model.files.current.buffer, preview_buffer),
        );

        let mut actions = Vec::new();

        if let Some(selected) = selection::get_current_selected_path(model) {
            tracing::trace!("loading selection: {:?}", selected);

            actions.push(Action::Load(
                WindowType::Preview,
                selected.to_path_buf(),
                None,
            ));
        }

        actions
    } else {
        Vec::new()
    }
}

fn mem_replace_buffer(mode: &Mode, dest: &mut Buffer, src: Buffer) -> Buffer {
    let mut result = mem::replace(dest, src);
    mem::swap(&mut dest.view_port, &mut result.view_port);

    yeet_buffer::update::update_buffer(mode, &mut result, &BufferMessage::UpdateViewPortByCursor);
    yeet_buffer::update::update_buffer(mode, dest, &BufferMessage::UpdateViewPortByCursor);

    result
}

