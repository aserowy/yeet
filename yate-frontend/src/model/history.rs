use std::{
    cmp::Reverse,
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
    time::{self, SystemTime},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default)]
pub struct History {
    entries: Vec<HistoryEntry>,
}

impl History {
    pub fn add(&mut self, path: PathBuf) {
        let timestamp = SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.entries.insert(
            0,
            HistoryEntry {
                added_at: timestamp,
                path,
                state: HistoryState::Added,
            },
        );
    }

    pub fn get_selection(&self, path: &PathBuf) -> Option<String> {
        self.entries
            .iter()
            .filter(|hstry| hstry.path.starts_with(path))
            .find_map(|hstry| {
                get_next(path, &hstry.path).map(|child| {
                    child
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap_or("")
                        .to_string()
                })
            })
    }

    // TODO: Error handling (all over the unwraps in yate!) and return Result here!
    pub fn load(&mut self) {
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

        for result in history_csv_reader.deserialize() {
            match result {
                Ok(entry) => self.entries.push(entry),
                Err(_) => {}
            };
        }

        self.entries
            .sort_unstable_by_key(|entry| Reverse(entry.added_at));
    }

    // TODO: Error handling (all over the unwraps in yate!) and return Result here!
    // TODO: Optimize history objects
    pub fn save(&self) {
        let entries: Vec<_> = self
            .entries
            .iter()
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
}

#[derive(Debug, Deserialize, Serialize)]
struct HistoryEntry {
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
