use std::path::PathBuf;

pub mod file;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FileRegister {
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
