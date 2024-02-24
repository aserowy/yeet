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
pub struct Register {
    current: CurrentRegister,
    pub path: PathBuf,
    trashed: Vec<RegisterEntry>,
    yanked: Option<RegisterEntry>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum CurrentRegister {
    _Custom(String),
    Trash,
    #[default]
    Yank,
}

impl Register {
    pub fn add_or_update(&mut self, path: &Path) -> Option<RegisterEntry> {
        if let Some((id, target)) = decompose_compression_path(path) {
            if self.yanked.as_ref().is_some_and(|entry| entry.id == id) {
                if let Some(entry) = self.yanked.as_mut() {
                    entry.status = RegisterStatus::Ready;
                }

                None
            } else if let Some(index) = self.trashed.iter().position(|entry| entry.id == id) {
                self.trashed[index].status = RegisterStatus::Ready;

                None
            } else {
                self.trashed.push(RegisterEntry {
                    cache: self.path.join(&id),
                    id,
                    status: RegisterStatus::Ready,
                    target,
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

    pub fn get(&self, register: &str) -> Option<RegisterEntry> {
        let entry = match register {
            "\"" => match self.current {
                CurrentRegister::Trash => self.trashed.first().cloned(),
                CurrentRegister::Yank => self.yanked.clone(),
                _ => None,
            },
            "0" => self.yanked.clone(),
            "1" => self.trashed.first().cloned(),
            "2" => self.trashed.get(1).cloned(),
            "3" => self.trashed.get(2).cloned(),
            "4" => self.trashed.get(3).cloned(),
            "5" => self.trashed.get(4).cloned(),
            "6" => self.trashed.get(5).cloned(),
            "7" => self.trashed.get(6).cloned(),
            "8" => self.trashed.get(7).cloned(),
            "9" => self.trashed.get(8).cloned(),
            // TODO: add custom register handling
            _ => None,
        };

        let is_ready = entry
            .as_ref()
            .is_some_and(|entry| entry.status == RegisterStatus::Ready);

        if is_ready {
            entry
        } else {
            None
        }
    }

    pub fn print(&self) -> Vec<String> {
        let mut contents = vec!["Name Content".to_string()];
        if let Some(current) = &self.get("\"") {
            contents.push(print_content("\"\"", current));
        }
        if let Some(yanked) = &self.yanked {
            contents.push(print_content("\"0", yanked));
        }
        for (index, entry) in self.trashed.iter().enumerate() {
            let register_name = format!("\"{}", index + 1);
            contents.push(print_content(&register_name, entry));
        }

        contents
    }

    pub fn remove(&mut self, path: &Path) {
        if let Some((id, _)) = decompose_compression_path(path) {
            let index = self.trashed.iter().position(|entry| entry.id == id);
            if let Some(index) = index {
                self.trashed.remove(index);
            }
        }
    }

    pub fn trash(&mut self, path: &Path) -> (RegisterEntry, Option<RegisterEntry>) {
        self.current = CurrentRegister::Trash;

        let entry = RegisterEntry::from(path, self);
        self.trashed.insert(0, entry.clone());

        let old_entry = if self.trashed.len() > 9 {
            self.trashed.pop()
        } else {
            None
        };

        (entry, old_entry)
    }

    pub fn yank(&mut self, path: &Path) -> (RegisterEntry, Option<RegisterEntry>) {
        self.current = CurrentRegister::Yank;

        let entry = RegisterEntry::from(path, self);
        (entry.clone(), self.yanked.replace(entry))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegisterEntry {
    pub id: String,
    pub cache: PathBuf,
    pub status: RegisterStatus,
    pub target: PathBuf,
}

impl RegisterEntry {
    fn from(path: &Path, register: &Register) -> Self {
        let id = compose_compression_name(path);

        Self {
            cache: register.path.join(&id),
            id,
            status: RegisterStatus::default(),
            target: path.to_path_buf(),
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

    if let Some(file_name) = entry.target.file_name() {
        let target_file = target_path.join(file_name);
        fs::rename(entry.target, target_file.clone()).await?;
        compress_with_archive_name(&target_file, &entry.id).await?;
    }

    fs::remove_dir_all(target_path).await?;

    Ok(())
}

pub async fn compress(entry: RegisterEntry) -> Result<(), AppError> {
    compress_with_archive_name(&entry.target, &entry.id).await
}

pub async fn delete(entry: RegisterEntry) -> Result<(), AppError> {
    let path = get_register_path().await?.join(&entry.id);
    fs::remove_file(path).await?;
    Ok(())
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

pub async fn init(register: &mut Register, emitter: &mut Emitter) -> Result<(), AppError> {
    register.path = get_register_path().await?;

    let mut read_dir = fs::read_dir(&register.path).await?;
    while let Some(entry) = read_dir.next_entry().await? {
        if let Some(old_entry) = register.add_or_update(&entry.path()) {
            emitter.run(Task::DeleteRegisterEntry(old_entry));
        }
    }

    emitter.watch(&register.path)?;

    Ok(())
}

pub fn restore(entry: RegisterEntry, path: PathBuf) -> Result<(), AppError> {
    let archive_file = File::open(entry.cache)?;
    let archive_decoder = GzDecoder::new(archive_file);
    let mut archive = Archive::new(archive_decoder);
    archive.unpack(path)?;

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

// TODO: add register name
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

// TODO: read register name
fn decompose_compression_path(path: &Path) -> Option<(String, PathBuf)> {
    if let Some(file_name) = path.file_name() {
        let file_name = file_name.to_string_lossy();
        if let Some((_, path_string)) = file_name.split_once('%') {
            let path = path_string.replace("%002F%", "/").replace("%0025%", "%");

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

fn print_content(register: &str, entry: &RegisterEntry) -> String {
    let content = match entry.status {
        RegisterStatus::Processing => "Processing".to_string(),
        RegisterStatus::Ready => entry.target.to_string_lossy().to_string(),
    };

    format!("{:<4} {}", register, content)
}

mod test {
    #[test]
    fn test_register_add_or_update() {
        use std::path::PathBuf;
        let mut register = super::Register {
            current: Default::default(),
            path: std::path::PathBuf::from("/some/path"),
            trashed: Vec::new(),
            yanked: None,
        };

        let path = PathBuf::from("/other/path/.direnv");
        let (entry, _) = register.trash(&path);

        assert_eq!(1, register.trashed.len());
        assert_eq!(super::RegisterStatus::Processing, entry.status);

        let path = PathBuf::from("/some/path").join(entry.id);
        register.add_or_update(&path);

        assert_eq!(1, register.trashed.len());
        assert_eq!(super::RegisterStatus::Ready, register.trashed[0].status);

        let id = "1708576379595%%002F%home%002F%user%002F%src%002F%yeet%002F%.direnv";
        let path = PathBuf::from("/some/path").join(id);
        register.add_or_update(&path);

        assert_eq!(2, register.trashed.len());
        assert_eq!(id, register.trashed[1].id);
        assert_eq!(super::RegisterStatus::Ready, register.trashed[1].status);

        let id_new = "2708576379595%%002F%home%002F%user%002F%src%002F%yeet%002F%.direnv";
        let path = PathBuf::from("/some/path").join(id_new);
        register.add_or_update(&path);

        assert_eq!(3, register.trashed.len());
        assert_eq!(id, register.trashed[2].id);
    }

    #[test]
    fn test_compose_decompose_compression_name() {
        // TODO: Check windows path format as well!

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
