use std::path::Path;

use ratatui::style::Color;
use yeet_buffer::model::{BufferLine, StylePartial, StylePartialSpan};
use yeet_keymap::message::ContentKind;

pub fn from(path: &Path) -> BufferLine {
    let content = match path.file_name() {
        Some(content) => content.to_str().unwrap_or("").to_string(),
        None => "".to_string(),
    };

    // TODO: Handle transition states like adding, removing, renaming
    let style = if path.is_dir() {
        let length = content.chars().count();
        vec![StylePartialSpan {
            end: length,
            style: StylePartial::Foreground(Color::LightBlue),
            ..Default::default()
        }]
    } else {
        vec![]
    };

    BufferLine {
        content,
        style,
        ..Default::default()
    }
}

pub fn from_enumeration(content: &String, kind: &ContentKind) -> BufferLine {
    // TODO: refactor with by path
    let style = if kind == &ContentKind::Directory {
        let length = content.chars().count();
        vec![StylePartialSpan {
            end: length,
            style: StylePartial::Foreground(Color::LightBlue),
            ..Default::default()
        }]
    } else {
        vec![]
    };

    BufferLine {
        content: content.to_string(),
        style,
        ..Default::default()
    }
}
