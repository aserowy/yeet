use std::path::PathBuf;

use yeet_buffer::model::viewport::WindowSettings;

#[derive(Debug)]
pub struct Settings {
    pub current: WindowSettings,
    pub parent: WindowSettings,
    pub preview: WindowSettings,
    pub selection_to_file_on_open: Option<PathBuf>,
    pub selection_to_stdout_on_open: bool,
    pub show_quickfix_signs: bool,
    pub show_mark_signs: bool,
    pub startup_path: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            current: WindowSettings {
                sign_column_width: 2,
            },
            parent: WindowSettings::default(),
            preview: WindowSettings::default(),
            selection_to_file_on_open: None,
            selection_to_stdout_on_open: false,
            show_mark_signs: true,
            show_quickfix_signs: true,
            startup_path: None,
        }
    }
}
