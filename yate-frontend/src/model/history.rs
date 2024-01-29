use std::{
    cmp::Reverse,
    collections::HashMap,
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
    time::{self},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default)]
pub struct History {
    pub entries: HashMap<String, Vec<HistoryEntry>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HistoryEntry {
    path: PathBuf,

    #[serde(skip)]
    state: HistoryState,
    added_at: u64,
}

#[derive(Debug, Default, PartialEq)]
enum HistoryState {
    Added,

    #[default]
    Loaded,
}

pub fn add(history: &mut History, path: &Path) {
    let entry = HistoryEntry {
        path: path.to_path_buf(),
        state: HistoryState::Added,
        added_at: time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    add_entry(history, entry);
}

pub fn get_selection(history: &History, path: &PathBuf) -> Option<String> {
    if let Some(path_last) = path.iter().last() {
        if let Some(path_name) = path_last.to_str() {
            if !history.entries.contains_key(path_name) {
                return None;
            }

            let entries: Vec<&HistoryEntry> = history.entries[path_name]
                .iter()
                .filter(|entry| entry.path.starts_with(path))
                .collect();

            return entries.iter().find_map(|hstry| {
                get_next(path, &hstry.path).map(|child| {
                    child
                        .file_name()
                        .and_then(|name| name.to_str())
                        .map(|name| name.to_string())
                        .unwrap_or_default()
                })
            });
        }
    }

    None
}

// TODO: Error handling (all over the unwraps in yate!) and return Result here!
pub fn load(history: &mut History) {
    let history_path = format!(
        "{}{}",
        dirs::cache_dir().unwrap().to_str().unwrap(),
        "/yate/.history"
    );

    if !Path::new(&history_path).exists() {
        return;
    }

    let history_file = File::open(history_path).unwrap();
    let mut history_csv_reader = csv::Reader::from_reader(history_file);

    history_csv_reader
        .deserialize()
        .flatten()
        .for_each(|entry| add_entry(history, entry));
}

// TODO: Error handling (all over the unwraps in yate!) and return Result here!
pub fn save(history: &History) {
    let entries: Vec<_> = history
        .entries
        .values()
        .flatten()
        .filter(|entry| entry.state == HistoryState::Added)
        .collect();

    let history_path = format!(
        "{}{}",
        dirs::cache_dir().unwrap().to_str().unwrap(),
        "/yate/.history"
    );

    if let Err(error) = fs::create_dir_all(Path::new(&history_path).parent().unwrap()) {
        print!("{}", error);
        return;
    }

    let history_writer = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(history_path)
        .unwrap();

    let mut history_csv_writer = csv::Writer::from_writer(history_writer);
    for entry in entries {
        history_csv_writer.serialize(entry).unwrap();
    }

    history_csv_writer.flush().unwrap();
}

fn add_entry(history: &mut History, entry: HistoryEntry) {
    if let Some(path_last) = entry.path.iter().last() {
        if let Some(path_name) = path_last.to_str() {
            if !history.entries.contains_key(path_name) {
                history.entries.insert(path_name.to_string(), Vec::new());
            }

            let entries = history.entries.get_mut(path_name).unwrap();
            entries.push(entry);
            entries.sort_by_key(|entry| Reverse(entry.added_at));
        }
    }
}

fn get_next(target: &PathBuf, history: &Path) -> Option<PathBuf> {
    let mut current = history;

    loop {
        match current.parent() {
            Some(parent) => {
                if parent == target {
                    if current.exists() {
                        return Some(current.to_path_buf());
                    } else {
                        return None;
                    }
                }

                current = parent;
            }
            None => return None,
        }
    }
}
