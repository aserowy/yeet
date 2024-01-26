use std::path::{Path, PathBuf};

use ratatui::{prelude::Rect, style::Color};
use yate_keymap::message::Message;

use crate::model::{
    buffer::{Buffer, BufferLine, StylePartial},
    Model,
};

use super::buffer;

pub fn update_buffer_with_path(buffer: &mut Buffer, layout: &Rect, message: &Message, path: &Path) {
    buffer.view_port.height = usize::from(layout.height);
    buffer.view_port.width = usize::from(layout.width);

    let content = if path.is_dir() {
        get_directory_content(path)
    } else {
        Vec::new()
    };

    buffer.lines = content;

    buffer::update(buffer, message);
}

pub fn get_target_path(model: &Model) -> Option<PathBuf> {
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

fn get_directory_content(path: &Path) -> Vec<BufferLine> {
    let mut content: Vec<_> = std::fs::read_dir(path)
        .unwrap()
        .map(|entry| get_bufferline_by_path(&entry.unwrap().path()))
        .collect();

    content.sort_unstable_by(|a, b| {
        a.content
            .to_ascii_uppercase()
            .cmp(&b.content.to_ascii_uppercase())
    });

    content
}

fn get_bufferline_by_path(path: &Path) -> BufferLine {
    let content = path.file_name().unwrap().to_str().unwrap().to_string();

    let style = if path.is_dir() {
        vec![(
            0,
            content.chars().count(),
            StylePartial::Foreground(Color::LightBlue),
        )]
    } else {
        vec![]
    };

    BufferLine {
        content,
        style,
        ..Default::default()
    }
}
