use std::path::PathBuf;

use ratatui::widgets::ListState;
use teywi_keymap::action::Mode;

#[derive(Debug)]
pub struct Model {
    pub current_directory: DirectoryListModel,
    pub current_path: PathBuf,
    pub key_sequence: String,
    pub mode: Mode,
    pub parent_directory: DirectoryListModel,
}

impl Default for Model {
    fn default() -> Self {
        return Model {
            current_path: PathBuf::from("/home/serowy/"),
            current_directory: DirectoryListModel::default(),
            key_sequence: String::new(),
            mode: Mode::default(),
            parent_directory: DirectoryListModel::default(),
        };
    }
}

#[derive(Debug, Default)]
pub struct DirectoryListModel {
    pub paths: Vec<PathBuf>,
    pub state: ListState,
}
