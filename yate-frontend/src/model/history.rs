use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    path::{Components, Path, PathBuf},
    time::{self},
};

#[derive(Debug, Default)]
pub struct History {
    pub entries: HashMap<String, HistoryNode>,
}

#[derive(Debug)]
pub struct HistoryNode {
    changed_at: u64,
    component: String,
    nodes: HashMap<String, HistoryNode>,
    state: HistoryState,
}

#[derive(Clone, Debug, Default, PartialEq)]
enum HistoryState {
    Added,

    #[default]
    Loaded,
}

// TODO: Error handling (all over the unwraps in yate!) and return Result here!
pub fn add(history: &mut History, path: &Path) {
    let added_at = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut iter = path.components();
    if let Some(component) = iter.next() {
        if let Some(component_name) = component.as_os_str().to_str() {
            add_entry(&mut history.entries, added_at, component_name, iter);
        }
    }
}

pub fn get_selection<'a>(history: &'a History, path: &Path) -> Option<&'a str> {
    let mut current_nodes = &history.entries;
    for component in path.components() {
        if let Some(current_name) = component.as_os_str().to_str() {
            if let Some(current_node) = current_nodes.get(current_name) {
                current_nodes = &current_node.nodes;
            } else {
                return None;
            }
        }
    }

    current_nodes
        .values()
        .max_by_key(|node| node.changed_at)
        .map(|node| node.component.as_str())
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
                add_entry(&mut history.entries, changed_at, component_name, iter);
            }
        }
    }
}

// TODO: Error handling (all over the unwraps in yate!) and return Result here!
pub fn save(history: &History) {
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
        .append(true)
        .open(history_path)
        .unwrap();

    let mut history_csv_writer = csv::Writer::from_writer(history_writer);
    for (changed_at, state, path) in entries {
        if state == HistoryState::Loaded {
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

fn add_entry(
    nodes: &mut HashMap<String, HistoryNode>,
    changed_at: u64,
    component_name: &str,
    mut component_iter: Components<'_>,
) {
    if !nodes.contains_key(component_name) {
        nodes.insert(
            component_name.to_string(),
            HistoryNode {
                changed_at,
                component: component_name.to_string(),
                nodes: HashMap::new(),
                state: HistoryState::Added,
            },
        );
    }

    if let Some(current_node) = nodes.get_mut(component_name) {
        if current_node.changed_at < changed_at {
            current_node.changed_at = changed_at;
            current_node.state = HistoryState::Added;
        }

        if let Some(next_component) = component_iter.next() {
            if let Some(next_component_name) = next_component.as_os_str().to_str() {
                add_entry(
                    &mut current_node.nodes,
                    changed_at,
                    next_component_name,
                    component_iter,
                );
            }
        }
    }
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
