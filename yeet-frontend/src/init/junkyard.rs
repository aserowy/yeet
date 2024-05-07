use std::{
    fs::File,
    path::{Path, PathBuf},
    time,
};

use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use tar::Archive;
use tokio::fs;

use crate::{
    error::AppError,
    event::Emitter,
    model::junkyard::{FileEntry, JunkYard},
    task::Task,
    update::junkyard::add_or_update_junkyard_entry,
};

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
        if let Some(obsolete) = add_or_update_junkyard_entry(junk, &entry.path()) {
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
