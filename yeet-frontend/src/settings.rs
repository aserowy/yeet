use std::path::PathBuf;

#[derive(Debug)]
pub struct Settings {
    pub current: Buffer,
    pub parent: Buffer,
    pub preview: Buffer,
    pub show_quickfix_signs: bool,
    pub show_mark_signs: bool,
    pub stdout_on_open: bool,
    pub startup_path: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            current: Buffer {
                sign_column_width: 2,
            },
            parent: Buffer::default(),
            preview: Buffer::default(),
            show_mark_signs: false,
            show_quickfix_signs: true,
            stdout_on_open: false,
            startup_path: None,
        }
    }
}

#[derive(Debug, Default)]
pub struct Buffer {
    pub sign_column_width: usize,
}
