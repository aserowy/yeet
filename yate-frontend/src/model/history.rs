use std::path::PathBuf;

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
                if let Some(child) = get_next(path, &hstry.path) {
                    Some(
                        child
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap_or("")
                            .to_string(),
                    )
                } else {
                    None
                }
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

fn get_next(target: &PathBuf, history: &PathBuf) -> Option<PathBuf> {
    let mut current = history.as_path();
    let mut child = history.as_path();

    loop {
        match current.parent() {
            Some(parent) => {
                if parent == target {
                    if child.exists() {
                        return Some(child.to_path_buf());
                    } else {
                        return None;
                    }
                }

                child = current;
                current = parent;
            }
            None => return None,
        }
    }
}
