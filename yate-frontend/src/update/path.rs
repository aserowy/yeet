use std::path::{Path, PathBuf};

use ratatui::style::Color;

use crate::{
    error::AppError,
    event::PostRenderAction,
    model::{
        buffer::{BufferLine, StylePartial},
        Model,
    },
};

pub fn get_bufferline_by_path(path: &Path) -> BufferLine {
    let content = match path.file_name() {
        Some(content) => content.to_str().unwrap_or("").to_string(),
        None => "".to_string(),
    };

    // TODO: Handle transition states like adding, removing, renaming
    let style = if path.is_dir() {
        let length = content.chars().count();
        vec![(0, length, StylePartial::Foreground(Color::LightBlue))]
    } else {
        vec![]
    };

    BufferLine {
        content,
        style,
        ..Default::default()
    }
}

pub fn get_directory_content(path: &Path) -> Result<Vec<BufferLine>, AppError> {
    match std::fs::read_dir(path) {
        Ok(content) => Ok(content
            .flatten()
            .map(|entry| get_bufferline_by_path(&entry.path()))
            .collect()),
        Err(error) => Err(AppError::FileOperationFailed(error)),
    }
}

pub fn get_selected_path(model: &Model) -> Option<PathBuf> {
    let buffer = &model.current.buffer;
    if buffer.lines.is_empty() {
        return None;
    }

    if let Some(cursor) = &buffer.cursor {
        let current = &buffer.lines[cursor.vertical_index];
        let target = model.current.path.join(&current.content);

        if target.exists() {
            Some(target)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn set_current_to_parent(model: &mut Model) -> Option<Vec<PostRenderAction>> {
    let mut actions = Vec::new();
    if let Some(parent) = model.current.path.parent() {
        if model.current.path == parent {
            return None;
        }

        let parent_parent = parent.parent();
        if let Some(parent) = parent_parent {
            actions.push(PostRenderAction::WatchPath(parent.to_path_buf()));
        }

        model.parent.path = parent_parent.map(|path| path.to_path_buf());
        model.current.path = parent.to_path_buf();
    }

    Some(actions)
}

pub fn set_current_to_path(model: &mut Model, path: &Path) -> Option<Vec<PostRenderAction>> {
    let mut actions = Vec::new();
    if path.exists() {
        let directory = if path.is_dir() {
            path.to_path_buf()
        } else {
            return None;
        };

        if let Some(parent) = &model.parent.path {
            actions.push(PostRenderAction::UnwatchPath(parent.clone()));
        }

        let parent_parent = directory.parent();
        if let Some(parent) = parent_parent {
            actions.push(PostRenderAction::WatchPath(parent.to_path_buf()));
        }
        model.parent.path = parent_parent.map(|path| path.to_path_buf());

        actions.push(PostRenderAction::UnwatchPath(model.current.path.clone()));
        actions.push(PostRenderAction::WatchPath(directory.clone()));
        model.current.path = directory;
    }

    Some(actions)
}

pub fn set_current_to_selected(model: &mut Model) -> Option<Vec<PostRenderAction>> {
    let mut actions = Vec::new();
    if let Some(selected) = get_selected_path(model) {
        if model.current.path == selected {
            return None;
        } else if !selected.is_dir() {
            return None;
        }

        if let Some(parent) = &model.parent.path {
            actions.push(PostRenderAction::UnwatchPath(parent.clone()));
        }
        model.parent.path = Some(model.current.path.clone());
        model.current.path = selected.to_path_buf();
    };

    Some(actions)
}

pub fn set_preview_to_selected(
    model: &mut Model,
    unwatch_old_path: bool,
    watch_new_path: bool,
) -> Option<Vec<PostRenderAction>> {
    let mut actions = Vec::new();
    if let Some(selected) = get_selected_path(model) {
        let current = &model.current.path;
        if current == &selected {
            return None;
        } else if model.preview.path == selected {
            return None;
        }

        if unwatch_old_path {
            actions.push(PostRenderAction::UnwatchPath(model.preview.path.clone()));
        }

        if watch_new_path {
            actions.push(PostRenderAction::WatchPath(selected.clone()));
        }

        model.preview.path = selected.to_path_buf();
    };

    Some(actions)
}
