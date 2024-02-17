use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct Settings {
    pub stdout_selection: bool,
    pub startup_path: Option<PathBuf>,
}
