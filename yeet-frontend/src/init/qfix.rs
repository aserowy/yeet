use std::{
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
};

use crate::{error::AppError, model::qfix::QuickFix};

#[tracing::instrument]
pub fn load_qfix_from_files(qfix: &mut QuickFix) -> Result<(), AppError> {
    let qfix_cache_path = get_qfix_cache_path()?;
    if !Path::new(&qfix_cache_path).exists() {
        tracing::debug!("qfix file does not exist on path {}", qfix_cache_path);

        return Ok(());
    }

    // TODO: change to tokio fs
    let qfix_cache_file = File::open(qfix_cache_path)?;
    let mut qfix_entry_csv_reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(qfix_cache_file);

    tracing::trace!("qfix file opened for reading");

    for result in qfix_entry_csv_reader.records() {
        let record = match result {
            Ok(record) => record,
            Err(_) => return Err(AppError::LoadQuickFixFailed),
        };

        let path = match record.get(0) {
            Some(path) => PathBuf::from(path),
            None => continue,
        };

        qfix.entries.push(path);
    }

    tracing::trace!("qfix file read");

    Ok(())
}

#[tracing::instrument]
pub fn save_qfix_to_files(qfix: &QuickFix) -> Result<(), AppError> {
    let qfix_entry_path = get_qfix_cache_path()?;
    let qfix_entry_dictionary = match Path::new(&qfix_entry_path).parent() {
        Some(path) => path,
        None => return Err(AppError::LoadQuickFixFailed),
    };

    fs::create_dir_all(qfix_entry_dictionary)?;

    let qfix_entry_writer = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(qfix_entry_path)?;

    tracing::trace!("qfix file opened for writing");

    let mut writer = csv::Writer::from_writer(qfix_entry_writer);
    for path in qfix.entries.iter() {
        if !path.exists() {
            continue;
        }

        if let Some(path) = path.to_str() {
            let write_result = writer.write_record([path]);
            if let Err(error) = write_result {
                tracing::error!("writing qfix entry failed: {:?}", error);
            }
        }
    }

    writer.flush()?;

    tracing::trace!("qfix file written");

    Ok(())
}

fn get_qfix_cache_path() -> Result<String, AppError> {
    let cache_dir = match dirs::cache_dir() {
        Some(cache_dir) => match cache_dir.to_str() {
            Some(cache_dir_string) => cache_dir_string.to_string(),
            None => return Err(AppError::LoadQuickFixFailed),
        },
        None => return Err(AppError::LoadQuickFixFailed),
    };

    Ok(format!("{}{}", cache_dir, "/yeet/qfix"))
}
