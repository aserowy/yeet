use std::path::Path;

use crate::model::Model;

// TODO: refactor into keymap crate
pub enum Message {
    Refresh,
}

pub fn update(state: &mut Model, message: &Message) {
    match message {
        Message::Refresh => {
            let path = Path::new(&state.current_path);
            let parent = path.parent().unwrap().as_os_str();

            state.parent_directory.paths = std::fs::read_dir(parent)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .collect();

            state.current_directory.paths = std::fs::read_dir(&state.current_path)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .collect();
        }
    }
}
