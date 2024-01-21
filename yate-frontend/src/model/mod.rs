use std::{env, path::PathBuf};

use ratatui::widgets::ListState;
use yate_keymap::action::Mode;

use self::buffer::Buffer;

pub mod buffer;

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
        let current_path = get_current_path();

        Self {
            current_path,
            current_directory: Buffer::default(),
            key_sequence: String::new(),
            mode: Mode::default(),
            parent_directory: DirectoryListModel::default(),
        }
    }
}

#[derive(Debug, Default)]
pub struct DirectoryListModel {
    pub paths: Vec<PathBuf>,
    pub state: ListState,
}

fn get_current_path() -> PathBuf {
    if let Ok(path) = env::current_dir() {
        return path;
    }

    dirs::home_dir().unwrap()
}
