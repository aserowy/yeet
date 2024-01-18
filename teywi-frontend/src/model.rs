use std::path::PathBuf;

use ratatui::widgets::ListState;
use teywi_keymap::ActionResolver;

#[derive(Debug)]
pub struct Model {
    pub action_resolver: ActionResolver,
    pub current_directory: DirectoryListModel,
    pub current_path: PathBuf,
    pub parent_directory: DirectoryListModel,
}

impl Default for Model {
    fn default() -> Self {
        return Model {
            action_resolver: ActionResolver::default(),
            current_path: PathBuf::from("/home/serowy/"),
            current_directory: DirectoryListModel::default(),
            parent_directory: DirectoryListModel::default(),
        };
    }
}

#[derive(Debug, Default)]
pub struct DirectoryListModel {
    pub paths: Vec<PathBuf>,
    pub state: ListState,
}
