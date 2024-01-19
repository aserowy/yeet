use std::path::Path;

use teywi_keymap::action::Action;

use crate::model::Model;

mod buffer;

pub fn update(model: &mut Model, message: &Action) {
    match message {
        Action::KeySequenceChanged(sequence) => {
            model.key_sequence = sequence.clone();
        }
        Action::ModeChanged(mode) => {
            model.mode = mode.clone();
        }
        Action::MoveCursorDown => {
            model.key_sequence = String::new();

            update_current_directory(model, message);
        }
        Action::Refresh => {
            update_current_directory(model, message);
            update_parent_directory(model);
        }
        Action::Quit => {}
    }
}

fn update_current_directory(model: &mut Model, message: &Action) {
    let path = Path::new(&model.current_path);

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
    let parent = path.parent().unwrap().as_os_str();

    model.parent_directory.paths = std::fs::read_dir(parent)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
}
