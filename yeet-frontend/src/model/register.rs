// see todo in current
// set watcher on dedicated dict for compresses

// softdelete/yank: add entry with state processing and invoke task

// handles are used in tasks: handles are pure functions defined here (equal to history/cache)
// handle delete: move path to cache, compress

// cleanup older files, thus all 10 registers are filled

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
pub struct Register {}

pub async fn compress(path: &Path) -> Result<(), AppError> {
    let target_path = get_register_path().await?.join(get_compression_name(path));
    let file = File::create(&target_path)?;
    let encoder = GzEncoder::new(file, Compression::default());
    let mut archive = tar::Builder::new(encoder);

    if path.is_dir() {
        archive.append_dir_all(".", path)?;
    } else {
        archive.append_file(".", &mut File::open(path)?)?;
    }

    Ok(())
}

fn get_compression_name(path: &Path) -> PathBuf {
    let added_at = match time::SystemTime::now().duration_since(time::UNIX_EPOCH) {
        Ok(time) => time.as_millis(),
        Err(_) => 0,
    };

    let path = path.to_string_lossy().replace("%", "%%").replace("/", "%");
    let file_name = format!("{}{}.tar.gz", added_at, path);

    Path::new(&file_name).to_path_buf()
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
