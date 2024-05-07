use std::{
    collections::HashMap,
    path::{Components, Path},
    time,
};

use crate::model::history::{History, HistoryNode, HistoryState};

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

pub fn add_history_component(
    nodes: &mut HashMap<String, HistoryNode>,
    changed_at: u64,
    state: HistoryState,
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
                state: state.clone(),
            },
        );
    }

    if let Some(current_node) = nodes.get_mut(component_name) {
        if current_node.changed_at < changed_at {
            current_node.changed_at = changed_at;
            current_node.state = state.clone();
        }

        if let Some(next_component) = component_iter.next() {
            if let Some(next_component_name) = next_component.as_os_str().to_str() {
                add_history_component(
                    &mut current_node.nodes,
                    changed_at,
                    state,
                    next_component_name,
                    component_iter,
                );
            }
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
