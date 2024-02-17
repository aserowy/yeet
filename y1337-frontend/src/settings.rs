use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct Settings {
    pub stdout_on_open: bool,
    pub startup_path: Option<PathBuf>,
}
