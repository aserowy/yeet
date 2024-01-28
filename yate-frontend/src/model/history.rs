use std::fs;
use std::io::Write;
use std::{
    fs::File,
    path::{Path, PathBuf},
};

#[derive(Debug, Default)]
pub struct History {
    entries: Vec<HistoryEntry>,
}

impl History {
    pub fn add(&mut self, path: PathBuf) {
        self.entries.insert(
            0,
            HistoryEntry {
                path,
                _state: HistoryState::Added,
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
    pub fn save(&self) {
        let paths: Vec<_> = self
            .entries
            .iter()
            .filter(|entry| entry._state == HistoryState::Added)
            .flat_map(|entry| entry.path.to_str())
            .collect();

        let serialized = serde_json::to_string(&paths).unwrap();
        let history_path = format!(
            "{}{}",
            dirs::cache_dir().unwrap().to_str().unwrap(),
            "/yate/.history"
        );

        if let Err(error) = fs::create_dir_all(Path::new(&history_path).parent().unwrap()) {
            print!("{}", error);
            return;
        }

        match File::create(history_path) {
            Ok(mut output) => write!(output, "{}", serialized).unwrap(),
            Err(error) => {
                print!("{}", error);
            }
        }
    }
}

#[derive(Debug)]
struct HistoryEntry {
    path: PathBuf,
    _state: HistoryState,
}

#[derive(Debug, PartialEq)]
enum HistoryState {
    Added,
    _Loaded,
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
