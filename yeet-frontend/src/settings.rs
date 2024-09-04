use std::path::PathBuf;

use yeet_buffer::model::BufferSettings;

pub struct Settings {
    pub current: BufferSettings,
    pub parent: BufferSettings,
    pub preview: BufferSettings,
    pub selection_to_file_on_open: Option<PathBuf>,
    pub selection_to_stdout_on_open: bool,
    pub show_quickfix_signs: bool,
    pub show_mark_signs: bool,
    pub startup_path: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            current: BufferSettings {
                sign_column_width: 2,
            },
            parent: BufferSettings::default(),
            preview: BufferSettings::default(),
            selection_to_file_on_open: None,
            selection_to_stdout_on_open: false,
            show_mark_signs: true,
            show_quickfix_signs: true,
            startup_path: None,
        }
    }
}
