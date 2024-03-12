use std::path::PathBuf;

use crate::error::AppError;

pub mod file;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct JunkYard {
    current: RegisterType,
    pub path: PathBuf,
    trashed: Vec<Transaction>,
    yanked: Option<Transaction>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum RegisterType {
    _Custom(String),
    Trash,
    #[default]
    Yank,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transaction {
    pub id: String,
    pub entries: Vec<FileEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileEntry {
    pub id: String,
    pub cache: PathBuf,
    pub status: RegisterStatus,
    pub target: PathBuf,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum RegisterStatus {
    #[default]
    Processing,
    Ready,
}

pub fn get_junkyard_path() -> Result<PathBuf, AppError> {
    let register_path = match dirs::cache_dir() {
        Some(cache_dir) => cache_dir.join("yeet/register/"),
        None => return Err(AppError::LoadHistoryFailed),
    };

    Ok(register_path)
}
