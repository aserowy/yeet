use std::{
    collections::HashMap,
    path::{Components, Path},
    time::{self},
};

pub mod cache;

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
pub enum HistoryState {
    Added,

    #[default]
    Loaded,
}

impl History {
    // TODO: Error handling (all over the unwraps in yate!) and return Result here!
    pub fn add(&mut self, path: &Path) {
        let added_at = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut iter = path.components();
        if let Some(component) = iter.next() {
            if let Some(component_name) = component.as_os_str().to_str() {
                add_entry(
                    &mut self.entries,
                    added_at,
                    HistoryState::Added,
                    component_name,
                    iter,
                );
            }
        }
    }

    pub fn get_selection<'a>(&'a self, path: &Path) -> Option<&'a str> {
        let mut current_nodes = &self.entries;
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
}

fn add_entry(
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
                add_entry(
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
