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

pub fn get_directory_content(path: &Path) -> Result<Vec<BufferLine>, AppError> {
    let mut content: Vec<_> = match std::fs::read_dir(path) {
        Ok(content) => content
            .flatten()
            .map(|entry| get_bufferline_by_path(&entry.path()))
            .collect(),
        Err(error) => return Err(AppError::FileOperationFailed(error)),
    };

    content.sort_unstable_by(|a, b| {
        a.content
            .to_ascii_uppercase()
            .cmp(&b.content.to_ascii_uppercase())
    });

    Ok(content)
}

fn get_bufferline_by_path(path: &Path) -> BufferLine {
    let content = match path.file_name() {
        Some(content) => content.to_str().unwrap_or("").to_string(),
        None => "".to_string(),
    };

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

        actions.push(PostRenderAction::WatchPath(parent.to_path_buf()));

        if let Some(selected) = get_selected_path(model) {
            actions.push(PostRenderAction::UnwatchPath(selected.clone()));
        }

        model.current.path = parent.to_path_buf();
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

        actions.push(PostRenderAction::WatchPath(selected.clone()));
        if let Some(parent) = model.current.path.parent() {
            actions.push(PostRenderAction::UnwatchPath(parent.to_path_buf()));
        }

        model.current.path = selected.to_path_buf();
    };

    Some(actions)
}

pub fn set_preview_to_selected(model: &mut Model) -> Option<Vec<PostRenderAction>> {
    let mut actions = Vec::new();
    if let Some(selected) = get_selected_path(model) {
        let current = &model.current.path;
        if current == &selected {
            return None;
        }

        actions.push(PostRenderAction::WatchPath(selected.clone()));
        actions.push(PostRenderAction::UnwatchPath(
            model.preview.path.to_path_buf(),
        ));

        model.preview.path = selected.to_path_buf();
    };

    Some(actions)
}
