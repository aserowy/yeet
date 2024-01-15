use std::path::PathBuf;

use ratatui::widgets::ListState;

#[derive(Debug)]
pub struct AppState {
    pub current_directory: PathBuf,
    pub current_directory_state: DirectoryListState,
    pub parent_directory_state: DirectoryListState,
}

impl Default for AppState {
    fn default() -> Self {
        return AppState {
            current_directory: PathBuf::from("/home/serowy/"),
            current_directory_state: DirectoryListState::default(),
            parent_directory_state: DirectoryListState::default(),
        };
    }
}

#[derive(Debug, Default)]
pub struct DirectoryListState {
    pub paths: Vec<PathBuf>,
    pub state: ListState,
}
