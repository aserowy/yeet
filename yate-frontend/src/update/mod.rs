use std::path::Path;

use yate_keymap::action::Action;

use crate::{layout::AppLayout, model::Model};

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
        }
        Action::Refresh => {
            update_current_directory(model, layout, message);
            update_parent_directory(model);
        }
        Action::SelectParent => {
            if let Some(parent) = &model.current_path.parent() {
                model.current_path = parent.to_path_buf();
            }

            update_current_directory(model, layout, message);
            update_parent_directory(model);
        }
        Action::Quit => {}
    }
}

fn update_current_directory(model: &mut Model, layout: &AppLayout, message: &Action) {
    let path = Path::new(&model.current_path);

    model.current_directory.view_port.height = usize::from(layout.current_directory.height);
    model.current_directory.view_port.width = usize::from(layout.current_directory.width);

    model.current_directory.lines = std::fs::read_dir(path)
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

    buffer::update(&mut model.current_directory, message);
}

fn update_parent_directory(model: &mut Model) {
    let path = Path::new(&model.current_path);

    match path.parent() {
        Some(parent) => {
            model.parent_directory.paths = std::fs::read_dir(parent)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .collect();
        }
        None => model.parent_directory.paths = vec![],
    }
}
