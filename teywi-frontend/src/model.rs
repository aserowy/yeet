use std::path::PathBuf;

use ratatui::widgets::ListState;
use teywi_keymap::action::Mode;

#[derive(Debug)]
pub struct Model {
    pub current_directory: Buffer,
    pub current_path: PathBuf,
    pub key_sequence: String,
    pub mode: Mode,
    pub parent_directory: DirectoryListModel,
}

impl Default for Model {
    fn default() -> Self {
        return Model {
            current_path: PathBuf::from("/home/serowy/"),
            current_directory: Buffer::default(),
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

#[derive(Debug, Default)]
pub struct Buffer {
    pub cursor: (CursorPosition, usize),
    pub lines: Vec<String>,
}

#[derive(Debug)]
pub enum CursorPosition {
    Absolute(usize),
    _End,
}

impl Default for CursorPosition {
    fn default() -> Self {
        CursorPosition::Absolute(0)
    }
}
