use std::{path::Path, time};

use crate::model::history::{add_history_component, History, HistoryState};

pub fn add_history_entry(history: &mut History, path: &Path) {
    let added_at = match time::SystemTime::now().duration_since(time::UNIX_EPOCH) {
        Ok(time) => time.as_secs(),
        Err(_) => 0,
    };

    let mut iter = path.components();
    if let Some(component) = iter.next() {
        if let Some(component_name) = component.as_os_str().to_str() {
            add_history_component(
                &mut history.entries,
                added_at,
                HistoryState::Added,
                component_name,
                iter,
            );
        }
    }
}

pub fn get_selection_from_history<'a>(history: &'a History, path: &Path) -> Option<&'a str> {
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
