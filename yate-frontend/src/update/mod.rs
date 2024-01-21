use std::path::{Path, PathBuf};

use ratatui::prelude::Rect;
use yate_keymap::action::Action;

use crate::{
    layout::AppLayout,
    model::{buffer::Buffer, Model},
};

mod buffer;

pub fn update(model: &mut Model, layout: &AppLayout, message: &Action) {
    match message {
        Action::ChangeKeySequence(sequence) => {
            model.key_sequence = sequence.clone();
        }
        Action::ChangeMode(mode) => {
            model.mode = mode.clone();
        }
        Action::MoveCursor(_) => {
            model.key_sequence = String::new();

            update_current_directory(model, layout, message);
            update_preview(model, layout, message);
        }
        Action::Refresh => {
            update_current_directory(model, layout, message);
            update_parent_directory(model, layout, message);
            update_preview(model, layout, message);
        }
        Action::SelectCurrent => {
            if let Some(target) = get_target_path(model) {
                if !target.is_dir() {
                    return;
                }

                model.current_path = target;

                update_current_directory(model, layout, message);
                update_parent_directory(model, layout, message);
                update_preview(model, layout, message);
            }
        }
        Action::SelectParent => {
            if let Some(parent) = &model.current_path.parent() {
                model.current_path = parent.to_path_buf();

                update_current_directory(model, layout, message);
                update_parent_directory(model, layout, message);
                update_preview(model, layout, message);
            }
        }
        Action::Quit => {}
    }
}

fn update_current_directory(model: &mut Model, layout: &AppLayout, message: &Action) {
    update_buffer_with_path(
        &mut model.current_directory,
        &layout.current_directory,
        message,
        &model.current_path,
    );
}

fn update_parent_directory(model: &mut Model, layout: &AppLayout, message: &Action) {
    let path = Path::new(&model.current_path);
    match path.parent() {
        Some(parent) => {
            update_buffer_with_path(
                &mut model.parent_directory,
                &layout.parent_directory,
                message,
                &parent,
            );
        }
        None => model.parent_directory.lines = vec![],
    }
}

fn update_preview(model: &mut Model, layout: &AppLayout, message: &Action) {
    if let Some(target) = get_target_path(model) {
        update_buffer_with_path(&mut model.preview, &layout.preview, message, &target);
    }
}

fn update_buffer_with_path(buffer: &mut Buffer, layout: &Rect, message: &Action, path: &Path) {
    buffer.view_port.height = usize::from(layout.height);
    buffer.view_port.width = usize::from(layout.width);

    let content = if path.is_dir() {
        get_directory_content(&path)
    } else {
        Vec::new()
    };

    buffer.lines = content;

    buffer::update(buffer, message);
}

fn get_target_path(model: &Model) -> Option<PathBuf> {
    let buffer = &model.current_directory;
    if let Some(cursor) = &buffer.cursor {
        let current = &buffer.lines[cursor.vertical_index];
        let target = model.current_path.join(current);

        if target.exists() {
            Some(target)
        } else {
            None
        }
    } else {
        None
    }
}

fn get_directory_content(path: &Path) -> Vec<String> {
    let mut content: Vec<_> = std::fs::read_dir(path)
        .unwrap()
        .map(|entry| {
            entry
                .unwrap()
                .path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
        })
        .collect();

    content.sort_unstable();

    content
}
