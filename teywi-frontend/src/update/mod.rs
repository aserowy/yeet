use std::path::Path;

use crate::model::Model;

pub enum Message {
    Refresh,
}

pub fn update(model: &mut Model, message: Message) {
    match message {
        Message::Refresh => {
            let path = Path::new(&model.current_path);
            let parent = path.parent().unwrap().as_os_str();

            model.parent_directory.paths = std::fs::read_dir(parent)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .collect();

            model.current_directory.paths = std::fs::read_dir(&model.current_path)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .collect();
        }
    }
}
