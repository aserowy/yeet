use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
};

use super::{History, HistoryNode, HistoryState};

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
    let mut history_csv_reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(history_file);

    for result in history_csv_reader.records() {
        let record = result.unwrap();
        let changed_at = record.get(0).unwrap().parse::<u64>().unwrap();
        let path = record.get(1).unwrap();

        let mut iter = Path::new(path).components();
        if let Some(component) = iter.next() {
            if let Some(component_name) = component.as_os_str().to_str() {
                super::add_entry(&mut history.entries, changed_at, component_name, iter);
            }
        }
    }
}

// TODO: Error handling (all over the unwraps in yate!) and return Result here!
pub fn optimize() {
    let mut history = History::default();
    load(&mut history);
    save_filtered(&history, HistoryState::Loaded, true);
}

pub fn save(history: &History) {
    save_filtered(history, HistoryState::Added, false);
}

// TODO: Error handling (all over the unwraps in yate!) and return Result here!
fn save_filtered(history: &History, state_filter: HistoryState, overwrite: bool) {
    let entries = get_paths(PathBuf::new(), &history.entries);

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
        .truncate(overwrite)
        .append(!overwrite)
        .open(history_path)
        .unwrap();

    let mut history_csv_writer = csv::Writer::from_writer(history_writer);
    for (changed_at, state, path) in entries {
        if state != state_filter {
            continue;
        }

        if !path.exists() {
            continue;
        }

        if let Some(path) = path.to_str() {
            history_csv_writer
                .write_record([changed_at.to_string().as_str(), path])
                .unwrap();
        }
    }

    history_csv_writer.flush().unwrap();
}

fn get_paths(
    current_path: PathBuf,
    nodes: &HashMap<String, HistoryNode>,
) -> Vec<(u64, HistoryState, PathBuf)> {
    let mut result = Vec::new();
    for node in nodes.values() {
        let mut path = current_path.clone();
        path.push(&node.component);

        if node.nodes.is_empty() {
            result.push((node.changed_at, node.state.clone(), path));
        } else {
            result.append(&mut get_paths(path, &node.nodes));
        }
    }

    result
}
