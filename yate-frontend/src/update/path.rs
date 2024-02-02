use std::path::{Path, PathBuf};

use ratatui::style::Color;

use crate::{
    error::AppError,
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

pub fn get_selected_path(model: &Model) -> Option<PathBuf> {
    let buffer = &model.current_directory;
    if buffer.lines.is_empty() {
        return None;
    }

    if let Some(cursor) = &buffer.cursor {
        let current = &buffer.lines[cursor.vertical_index];
        let target = model.current_path.join(&current.content);

        if target.exists() {
            Some(target)
        } else {
            None
        }
    } else {
        None
    }
}

fn get_bufferline_by_path(path: &Path) -> BufferLine {
    let content = path.file_name().unwrap().to_str().unwrap_or("").to_string();

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
