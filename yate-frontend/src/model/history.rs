use std::path::{Path, PathBuf};

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
