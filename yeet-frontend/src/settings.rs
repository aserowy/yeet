use std::path::PathBuf;

use yeet_buffer::model::BufferSettings;

#[derive(Debug)]
pub struct Settings {
    pub current: BufferSettings,
    pub parent: BufferSettings,
    pub preview: BufferSettings,
    pub show_quickfix_signs: bool,
    pub show_mark_signs: bool,
    pub stdout_on_open: bool,
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
            show_mark_signs: false,
            show_quickfix_signs: true,
            stdout_on_open: false,
            startup_path: None,
        }
    }
}
