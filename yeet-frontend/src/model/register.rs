// init register on start and delete entries > 10

// :reg .. expand commandline and exit on enter
// change state to echo path

use std::{
    cmp::Reverse,
    fs::File,
    path::{Path, PathBuf},
    time,
};

use flate2::{write::GzEncoder, Compression};
use tokio::fs;

use crate::error::AppError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Register {
    pub path: PathBuf,
    entries: Vec<RegisterEntry>,
}

impl Register {
    pub async fn new() -> Result<Self, AppError> {
        Ok(Self {
            path: get_register_path().await?,
            entries: Vec::new(),
        })
    }

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

    pub fn add_or_update(&mut self, path: &Path) {
        if let Some((id, target)) = decompose_compression_path(path) {
            let index = self.entries.iter().position(|entry| entry.id == id);
            if let Some(index) = index {
                self.entries[index].status = RegisterStatus::Ready;
            } else {
                self.entries.push(RegisterEntry {
                    id,
                    path: target,
                    status: RegisterStatus::Ready,
                });

                self.entries
                    .sort_unstable_by_key(|entry| Reverse(entry.id.clone()));
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegisterEntry {
    pub id: String,
    pub path: PathBuf,
    pub status: RegisterStatus,
}

impl RegisterEntry {
    fn from(path: &Path) -> Self {
        Self {
            id: compose_compression_name(path),
            path: path.to_path_buf(),
            status: RegisterStatus::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum RegisterStatus {
    #[default]
    Processing,

    Ready,
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
    let file = File::create(target_path)?;
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

fn compose_compression_name(path: &Path) -> String {
    let added_at = match time::SystemTime::now().duration_since(time::UNIX_EPOCH) {
        Ok(time) => time.as_millis(),
        Err(_) => 0,
    };

    let path = path
        .to_string_lossy()
        .replace('%', "%0025%")
        .replace('/', "%002F%");

    let file_name = format!("{}%{}", added_at, path);

    file_name
}

fn decompose_compression_path(path: &Path) -> Option<(String, PathBuf)> {
    if let Some(file_name) = path.file_name() {
        let file_name = file_name.to_string_lossy();
        if let Some((_, path_string)) = file_name.split_once('%') {
            let path = path_string.replace("%0025%", "%").replace("%002F%", "/");

            Some((file_name.to_string(), PathBuf::from(path)))
        } else {
            None
        }
    } else {
        None
    }
}

async fn get_register_cache_path() -> Result<PathBuf, AppError> {
    let cache_path = get_register_path().await?.join(".cache/");
    if !cache_path.exists() {
        fs::create_dir_all(&cache_path).await?;
    }
    Ok(cache_path)
}

pub async fn get_register_path() -> Result<PathBuf, AppError> {
    let register_path = match dirs::cache_dir() {
        Some(cache_dir) => cache_dir.join("yeet/register/"),
        None => return Err(AppError::LoadHistoryFailed),
    };

    if !register_path.exists() {
        fs::create_dir_all(&register_path).await?;
    }
    Ok(register_path)
}

mod test {
    #[test]
    fn test_register_add_or_update() {
        use std::path::PathBuf;
        let mut register = super::Register {
            path: std::path::PathBuf::from("/some/path"),
            entries: Vec::new(),
        };

        let path = PathBuf::from("/other/path/.direnv");
        let (entry, _) = register.add(&path);

        assert_eq!(1, register.entries.len());
        assert_eq!(super::RegisterStatus::Processing, entry.status);

        let path = PathBuf::from("/some/path").join(entry.id);
        register.add_or_update(&path);

        assert_eq!(1, register.entries.len());
        assert_eq!(super::RegisterStatus::Ready, register.entries[0].status);

        let id = "1708576379595%%002F%home%002F%user%002F%src%002F%yeet%002F%.direnv";
        let path = PathBuf::from("/some/path").join(id);
        register.add_or_update(&path);

        assert_eq!(2, register.entries.len());
        assert_eq!(id, register.entries[1].id);
        assert_eq!(super::RegisterStatus::Ready, register.entries[1].status);

        let id_new = "2708576379595%%002F%home%002F%user%002F%src%002F%yeet%002F%.direnv";
        let path = PathBuf::from("/some/path").join(id_new);
        register.add_or_update(&path);

        assert_eq!(3, register.entries.len());
        assert_eq!(id, register.entries[2].id);
    }

    #[test]
    fn test_compression_name_compose_decompose() {
        let path = std::path::Path::new("/home/U0025/sr%/y%et/%direnv");
        let name = super::compose_compression_name(&path);

        let composed = std::path::Path::new("/some/cache/register/").join(name.clone());
        let (dec_name, dec_path) = super::decompose_compression_path(&composed).unwrap();

        assert_eq!(name, dec_name);
        assert_eq!(path, dec_path);

        let path = std::path::Path::new("/home/user/sr%/y%et/%direnv");
        let name = super::compose_compression_name(&path);

        let composed = std::path::Path::new("/some/cache/register/").join(name.clone());
        let (dec_name, dec_path) = super::decompose_compression_path(&composed).unwrap();

        assert_eq!(name, dec_name);
        assert_eq!(path, dec_path);

        let path = std::path::Path::new("/home/user/src/yeet/.direnv");
        let name = super::compose_compression_name(&path);

        let composed = std::path::Path::new("/some/cache/register/").join(name.clone());
        let (dec_name, dec_path) = super::decompose_compression_path(&composed).unwrap();

        assert_eq!(name, dec_name);
        assert_eq!(path, dec_path);
    }
}
