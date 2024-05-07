use std::path::PathBuf;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct JunkYard {
    pub current: FileEntryType,
    pub path: PathBuf,
    pub trashed: Vec<FileTransaction>,
    pub yanked: Option<FileTransaction>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum FileEntryType {
    _Custom(String),
    #[default]
    Trash,
    Yank,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileTransaction {
    pub id: String,
    pub entries: Vec<FileEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileEntry {
    pub id: String,
    pub cache: PathBuf,
    pub status: FileEntryStatus,
    pub target: PathBuf,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum FileEntryStatus {
    #[default]
    Processing,
    Ready,
}
