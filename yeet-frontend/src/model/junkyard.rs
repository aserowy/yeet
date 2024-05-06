use std::{
    cmp::Reverse,
    fs::File,
    path::{Path, PathBuf},
    time,
};

use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use tar::Archive;
use tokio::fs;

use crate::{error::AppError, event::Emitter, task::Task};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct JunkYard {
    pub current: FileEntryType,
    pub path: PathBuf,
    pub trashed: Vec<FileTransaction>,
    pub yanked: Option<FileTransaction>,
}

// TODO: move all methods to update mod as pure functions
impl JunkYard {
    pub fn add_or_update(&mut self, path: &Path) -> Option<FileTransaction> {
        if let Some((id, file, target)) = decompose_compression_path(path) {
            if self.yanked.as_ref().is_some_and(|entry| entry.id == id) {
                if let Some(transaction) = self.yanked.as_mut() {
                    let entry = transaction
                        .entries
                        .iter_mut()
                        .find(|entry| entry.id == file);

                    if let Some(entry) = entry {
                        entry.status = FileEntryStatus::Ready;
                    } else {
                        let mut entry = FileEntry::from(id.to_string(), &target, &self.path);
                        entry.status = FileEntryStatus::Ready;
                        transaction.entries.push(entry);
                    }
                }

                None
            } else if let Some(index) = self.trashed.iter().position(|entry| entry.id == id) {
                let transaction = &mut self.trashed[index];
                let entry = transaction
                    .entries
                    .iter_mut()
                    .find(|entry| entry.id == file);

                if let Some(entry) = entry {
                    entry.status = FileEntryStatus::Ready;
                } else {
                    let mut entry = FileEntry::from(id.to_string(), &target, &self.path);
                    entry.status = FileEntryStatus::Ready;
                    transaction.entries.push(entry);
                }

                None
            } else {
                let mut entry = FileEntry::from(id.to_string(), &target, &self.path);
                entry.status = FileEntryStatus::Ready;

                self.trashed.push(FileTransaction {
                    id: id.to_owned(),
                    entries: vec![entry],
                });

                self.trashed
                    .sort_unstable_by_key(|entry| Reverse(entry.id.clone()));

                if self.trashed.len() > 9 {
                    self.trashed.pop()
                } else {
                    None
                }
            }
        } else {
            None
        }
    }

    pub fn get(&self, junk: &char) -> Option<&FileTransaction> {
        let transaction = match junk {
            '"' => match self.current {
                FileEntryType::Trash => self.trashed.first(),
                FileEntryType::Yank => self.yanked.as_ref(),
                FileEntryType::_Custom(_) => None,
            },
            '0' => self.yanked.as_ref(),
            '1' => self.trashed.first(),
            '2' => self.trashed.get(1),
            '3' => self.trashed.get(2),
            '4' => self.trashed.get(3),
            '5' => self.trashed.get(4),
            '6' => self.trashed.get(5),
            '7' => self.trashed.get(6),
            '8' => self.trashed.get(7),
            '9' => self.trashed.get(8),
            // TODO: add custom junk handling
            _ => None,
        };

        let is_ready = transaction.as_ref().is_some_and(|trnsctn| {
            trnsctn
                .entries
                .iter()
                .all(|entry| entry.status == FileEntryStatus::Ready)
        });

        if is_ready {
            transaction
        } else {
            None
        }
    }

    pub fn remove(&mut self, path: &Path) {
        if let Some((id, _, _)) = decompose_compression_path(path) {
            let index = self.trashed.iter().position(|entry| entry.id == id);
            if let Some(index) = index {
                self.trashed.remove(index);
            }
        }
    }

    pub fn trash(&mut self, paths: Vec<PathBuf>) -> (FileTransaction, Option<FileTransaction>) {
        self.current = FileEntryType::Trash;

        let transaction = FileTransaction::from(paths, self);
        self.trashed.insert(0, transaction.clone());

        let obsolete = if self.trashed.len() > 9 {
            self.trashed.pop()
        } else {
            None
        };

        (transaction, obsolete)
    }

    pub fn yank(&mut self, paths: Vec<PathBuf>) -> (FileTransaction, Option<FileTransaction>) {
        self.current = FileEntryType::Yank;

        let transaction = FileTransaction::from(paths, self);
        (transaction.clone(), self.yanked.replace(transaction))
    }
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

impl FileTransaction {
    fn from(paths: Vec<PathBuf>, junk: &JunkYard) -> Self {
        let added_at = match time::SystemTime::now().duration_since(time::UNIX_EPOCH) {
            Ok(time) => time.as_millis(),
            Err(_) => 0,
        };

        let entries = paths
            .into_iter()
            .map(|path| FileEntry::from(added_at.to_string(), &path, &junk.path))
            .collect();

        Self {
            id: added_at.to_string(),
            entries,
        }
    }
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

impl FileEntry {
    fn from(id: String, path: &Path, cache: &Path) -> Self {
        let id = compose_compression_name(id, path);

        Self {
            cache: cache.join(&id),
            id,
            status: FileEntryStatus::default(),
            target: path.to_path_buf(),
        }
    }
}

pub fn get_junkyard_path() -> Result<PathBuf, AppError> {
    let yard_dir = match dirs::cache_dir() {
        Some(cache_dir) => cache_dir.join("yeet/junkyard/"),
        None => return Err(AppError::LoadHistoryFailed),
    };

    Ok(yard_dir)
}

pub async fn cache_and_compress(entry: FileEntry) -> Result<(), AppError> {
    let cache_path = get_junk_cache_path().await?;

    let added_at = match time::SystemTime::now().duration_since(time::UNIX_EPOCH) {
        Ok(time) => time.as_nanos(),
        Err(_) => 0,
    };

    let target_path = cache_path.join(format!("{}/", added_at));
    if !target_path.exists() {
        fs::create_dir_all(&target_path).await?;
    }

    if let Some(file_name) = entry.target.file_name() {
        let target_file = target_path.join(file_name);
        fs::rename(entry.target, target_file.clone()).await?;
        compress_with_archive_name(&target_file, &entry.id).await?;
    }

    fs::remove_dir_all(target_path).await?;

    Ok(())
}

pub async fn compress(entry: FileEntry) -> Result<(), AppError> {
    compress_with_archive_name(&entry.target, &entry.id).await
}

pub async fn delete(entry: FileEntry) -> Result<(), AppError> {
    let path = get_junk_path().await?.join(&entry.id);
    fs::remove_file(path).await?;
    Ok(())
}

pub async fn init_junkyard(junk: &mut JunkYard, emitter: &mut Emitter) -> Result<(), AppError> {
    junk.path = get_junk_path().await?;

    let mut read_dir = fs::read_dir(&junk.path).await?;
    while let Some(entry) = read_dir.next_entry().await? {
        if let Some(obsolete) = junk.add_or_update(&entry.path()) {
            for entry in obsolete.entries {
                emitter.run(Task::DeleteJunkYardEntry(entry));
            }
        }
    }

    emitter.watch(&junk.path)?;

    Ok(())
}

pub fn restore(entry: FileEntry, path: PathBuf) -> Result<(), AppError> {
    let archive_file = File::open(entry.cache)?;
    let archive_decoder = GzDecoder::new(archive_file);
    let mut archive = Archive::new(archive_decoder);
    archive.unpack(path)?;

    Ok(())
}

async fn compress_with_archive_name(path: &Path, archive_name: &str) -> Result<(), AppError> {
    let compress_path = get_junk_compress_path().await?.join(archive_name);

    let file = File::create(&compress_path)?;
    let encoder = GzEncoder::new(file, Compression::default());
    let mut archive = tar::Builder::new(encoder);

    if let Some(file_name) = path.file_name() {
        if path.is_dir() {
            let archive_directory = format!("{}/", file_name.to_string_lossy());
            archive.append_dir_all(archive_directory, path)?;
        } else {
            archive.append_file(
                file_name.to_string_lossy().to_string(),
                &mut File::open(path)?,
            )?;
        }
    }
    archive.finish()?;

    let target_path = get_junk_path().await?.join(archive_name);
    match fs::rename(compress_path, target_path).await {
        Ok(it) => it,
        Err(err) => {
            tracing::error!("Failed to rename file: {:?}", err);
        }
    };

    Ok(())
}

fn compose_compression_name(id: String, path: &Path) -> String {
    let path = path
        .to_string_lossy()
        .replace('%', "%0025%")
        .replace('/', "%002F%");

    let file_name = format!("{}%{}", id, path);

    file_name
}

fn decompose_compression_path(path: &Path) -> Option<(String, String, PathBuf)> {
    if let Some(file_name) = path.file_name() {
        let file_name = file_name.to_string_lossy();
        if let Some((id, path_string)) = file_name.split_once('%') {
            let path = path_string.replace("%002F%", "/").replace("%0025%", "%");
            Some((id.to_string(), file_name.to_string(), PathBuf::from(path)))
        } else {
            None
        }
    } else {
        None
    }
}

async fn get_junk_cache_path() -> Result<PathBuf, AppError> {
    let path = get_junk_path().await?.join(".cache/");
    if !path.exists() {
        fs::create_dir_all(&path).await?;
    }
    Ok(path)
}

async fn get_junk_compress_path() -> Result<PathBuf, AppError> {
    let path = get_junk_path().await?.join(".compress/");
    if !path.exists() {
        fs::create_dir_all(&path).await?;
    }
    Ok(path)
}

async fn get_junk_path() -> Result<PathBuf, AppError> {
    let junk_path = get_junkyard_path()?;
    if !junk_path.exists() {
        fs::create_dir_all(&junk_path).await?;
    }
    Ok(junk_path)
}

mod test {
    #[test]
    fn junk_add_or_update() {
        use std::path::PathBuf;
        let mut junk = super::JunkYard {
            current: Default::default(),
            path: std::path::PathBuf::from("/some/path"),
            trashed: Vec::new(),
            yanked: None,
        };

        let path = PathBuf::from("/other/path/.direnv");
        let (transaction, _) = junk.trash(vec![path]);

        assert_eq!(1, junk.trashed.len());
        assert_eq!(
            super::FileEntryStatus::Processing,
            transaction.entries[0].status
        );

        let id = transaction.id + "%%002F%other%002F%path%002F%.direnv";
        let path = PathBuf::from("/some/path").join(id);
        junk.add_or_update(&path);

        assert_eq!(1, junk.trashed.len());
        assert_eq!(
            super::FileEntryStatus::Ready,
            junk.trashed[0].entries[0].status
        );

        let transaction = "1708576379595";
        let id = transaction.to_owned() + "%%002F%home%002F%user%002F%src%002F%yeet%002F%.direnv";
        let path = PathBuf::from("/some/path").join(id);
        junk.add_or_update(&path);

        assert_eq!(2, junk.trashed.len());
        assert_eq!(transaction, junk.trashed[1].id);
        assert_eq!(
            super::FileEntryStatus::Ready,
            junk.trashed[1].entries[0].status
        );

        let file = "%%002F%home%002F%user%002F%src%002F%yeet%002F%awesome".to_owned();
        let id = transaction.to_owned() + &file;
        let path = PathBuf::from("/some/path").join(&id);
        junk.add_or_update(&path);

        assert_eq!(2, junk.trashed.len());
        assert_eq!(transaction, junk.trashed[1].id);
        assert_eq!(id, junk.trashed[1].entries[1].id);
        assert_eq!(
            super::FileEntryStatus::Ready,
            junk.trashed[1].entries[1].status
        );

        let id_new = "2708576379595%%002F%home%002F%user%002F%src%002F%yeet%002F%.direnv";
        let path = PathBuf::from("/some/path").join(id_new);
        junk.add_or_update(&path);

        assert_eq!(3, junk.trashed.len());
        assert_eq!(transaction, junk.trashed[2].id);
    }

    #[test]
    fn compose_decompose_compression_name() {
        // TODO: Check windows path format as well!
        let id = "1708576379595".to_string();

        let path = std::path::Path::new("/home/U0025/sr%/y%et/%direnv");
        let name = super::compose_compression_name(id.to_string(), path);

        let composed = std::path::Path::new("/some/cache/junk/").join(name.clone());
        let (dec_id, dec_name, dec_path) = super::decompose_compression_path(&composed).unwrap();

        assert_eq!(id.to_string(), dec_id);
        assert_eq!(name, dec_name);
        assert_eq!(path, dec_path);

        let path = std::path::Path::new("/home/user/sr%/y%et/%direnv");
        let name = super::compose_compression_name(id.to_string(), path);

        let composed = std::path::Path::new("/some/cache/junk/").join(name.clone());
        let (dec_id, dec_name, dec_path) = super::decompose_compression_path(&composed).unwrap();

        assert_eq!(id.to_string(), dec_id);
        assert_eq!(name, dec_name);
        assert_eq!(path, dec_path);

        let path = std::path::Path::new("/home/user/src/yeet/.direnv");
        let name = super::compose_compression_name(id.to_string(), path);

        let composed = std::path::Path::new("/some/cache/junk/").join(name.clone());
        let (dec_id, dec_name, dec_path) = super::decompose_compression_path(&composed).unwrap();

        assert_eq!(id.to_string(), dec_id);
        assert_eq!(name, dec_name);
        assert_eq!(path, dec_path);
    }
}
