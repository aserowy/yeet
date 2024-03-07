use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
};

use crate::error::AppError;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Marks {
    pub entries: HashMap<char, PathBuf>,
}

#[tracing::instrument]
pub fn load(mark: &mut Marks) -> Result<(), AppError> {
    let mark_path = get_mark_path()?;
    if !Path::new(&mark_path).exists() {
        tracing::debug!("marks file does not exist on path {}", mark_path);

        return Ok(());
    }

    // TODO: change to tokio fs
    let mark_file = File::open(mark_path)?;
    let mut mark_csv_reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(mark_file);

    tracing::trace!("marks file opened for reading");

    for result in mark_csv_reader.records() {
        let record = match result {
            Ok(record) => record,
            Err(_) => return Err(AppError::LoadMarkFailed),
        };

        let char = match record.get(0) {
            Some(val) => {
                if let Ok(it) = val.parse::<char>() {
                    it
                } else {
                    continue;
                }
            }
            None => continue,
        };

        let path = match record.get(1) {
            Some(path) => PathBuf::from(path),
            None => continue,
        };

        if path.exists() {
            mark.entries.insert(char, path);
        }
    }

    tracing::trace!("marks file read");

    Ok(())
}

#[tracing::instrument]
pub fn save(marks: &Marks) -> Result<(), AppError> {
    let mark_path = get_mark_path()?;
    let mark_dictionary = match Path::new(&mark_path).parent() {
        Some(path) => path,
        None => return Err(AppError::LoadMarkFailed),
    };

    fs::create_dir_all(mark_dictionary)?;

    let mark_writer = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(mark_path)?;

    tracing::trace!("marks file opened for writing");

    let mut persisted = Marks::default();
    load(&mut persisted)?;
    persisted.entries.extend(marks.entries.clone());

    tracing::trace!("persisted marks loaded and merged");

    let mut writer = csv::Writer::from_writer(mark_writer);
    for (char, path) in marks.entries.iter() {
        if !path.exists() {
            continue;
        }

        if let Some(path) = path.to_str() {
            let write_result = writer.write_record([char.to_string().as_str(), path]);
            if let Err(error) = write_result {
                tracing::error!("writing mark failed: {:?}", error);
            }
        }
    }

    writer.flush()?;

    tracing::trace!("marks file written");

    Ok(())
}

fn get_mark_path() -> Result<String, AppError> {
    let cache_dir = match dirs::cache_dir() {
        Some(cache_dir) => match cache_dir.to_str() {
            Some(cache_dir_string) => cache_dir_string.to_string(),
            None => return Err(AppError::LoadMarkFailed),
        },
        None => return Err(AppError::LoadMarkFailed),
    };

    Ok(format!("{}{}", cache_dir, "/yeet/marks"))
}
