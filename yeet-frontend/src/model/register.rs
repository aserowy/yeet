// see todo in current
// set watcher on dedicated dict for compresses

// softdelete/yank: add entry with state processing and invoke task

// refresh registers on notify
// change state to echo path

// :reg .. expand commandline and exit on enter

use std::{
    fs::File,
    path::{Path, PathBuf},
    time,
};

use flate2::{write::GzEncoder, Compression};
use tokio::fs;

use crate::error::AppError;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Register {
    entries: Vec<RegisterEntry>,
}

impl Register {
    pub fn add(&mut self, path: &Path) -> (RegisterEntry, Option<RegisterEntry>) {
        let entry = RegisterEntry::from(path);
        self.entries.insert(0, entry.clone());

        let old_entry = if self.entries.len() > 10 {
            self.entries.pop()
        } else {
            None
        };

        (entry, old_entry)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RegisterEntry {
    pub id: String,
    pub path: PathBuf,
    pub status: RegisterStatus,
}

impl RegisterEntry {
    fn from(path: &Path) -> Self {
        Self {
            id: get_compression_name(&path),
            path: path.to_path_buf(),
            status: RegisterStatus::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum RegisterStatus {
    #[default]
    Processing,

    _Ready,
}

pub async fn cache_and_compress(entry: RegisterEntry) -> Result<(), AppError> {
    let cache_path = get_register_cache_path().await?;

    let added_at = match time::SystemTime::now().duration_since(time::UNIX_EPOCH) {
        Ok(time) => time.as_nanos(),
        Err(_) => 0,
    };

    let target_path = cache_path.join(format!("{}/", added_at));
    if !target_path.exists() {
        fs::create_dir_all(&target_path).await?;
    }

    if let Some(file_name) = entry.path.file_name() {
        let target_file = target_path.join(file_name);
        fs::rename(entry.path, target_file.clone()).await?;
        compress_with_archive_name(&target_file, &entry.id).await?;
    }

    fs::remove_dir_all(target_path).await?;

    Ok(())
}

pub async fn compress(entry: RegisterEntry) -> Result<(), AppError> {
    compress_with_archive_name(&entry.path, &entry.id).await
}

pub async fn delete(entry: RegisterEntry) -> Result<(), AppError> {
    let path = get_register_path().await?.join(&entry.id);
    fs::remove_file(path).await?;
    Ok(())
}

async fn compress_with_archive_name(path: &Path, archive_name: &str) -> Result<(), AppError> {
    let target_path = get_register_path().await?.join(archive_name);
    let file = File::create(&target_path)?;
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
    Ok(())
}

fn get_compression_name(path: &Path) -> String {
    let added_at = match time::SystemTime::now().duration_since(time::UNIX_EPOCH) {
        Ok(time) => time.as_millis(),
        Err(_) => 0,
    };

    let path = path.to_string_lossy().replace("%", "%%").replace("/", "%");
    let file_name = format!("{}{}", added_at, path);

    file_name
}

async fn get_register_cache_path() -> Result<PathBuf, AppError> {
    let cache_path = get_register_path().await?.join(".cache/");
    if !cache_path.exists() {
        fs::create_dir_all(&cache_path).await?;
    }
    Ok(cache_path)
}

async fn get_register_path() -> Result<PathBuf, AppError> {
    let register_path = match dirs::cache_dir() {
        Some(cache_dir) => cache_dir.join("yeet/register/"),
        None => return Err(AppError::LoadHistoryFailed),
    };

    if !register_path.exists() {
        fs::create_dir_all(&register_path).await?;
    }
    Ok(register_path)
}
