use std::{collections::HashMap, slice::Iter};

use crate::{
    action::{Action, Mode},
    key::Key,
};

#[derive(Debug)]
pub struct KeyTree {
    modes: HashMap<Mode, HashMap<Key, Node>>,
}

#[derive(Clone, Debug)]
pub enum Node {
    Key(HashMap<Key, Node>),
    Action(Action),
}

impl KeyTree {
    pub fn add_mapping(&mut self, mode: &Mode, keys: Vec<Key>, action: Action) {
        if !self.modes.contains_key(mode) {
            self.modes.insert(mode.clone(), HashMap::new());
        }

        let mut key_iter = keys.iter();
        if let Some(key) = key_iter.next() {
            add_mapping_node(
                self.modes.get_mut(mode).unwrap(),
                key,
                &mut key_iter,
                action,
            );
        }
    }

    pub fn get_node(&self, mode: &Mode, keys: &Vec<Key>) -> Option<Node> {
        if let Some(node) = self.modes.get(mode) {
            let mut key_iter = keys.iter();
            if let Some(key) = key_iter.next() {
                return get_node_from_tree(node, key, &mut key_iter);
            }
        }

        None
    }

    pub fn new() -> Self {
        Self {
            modes: HashMap::default(),
        }
    }
}

fn add_mapping_node(
    nodes: &mut HashMap<Key, Node>,
    key: &Key,
    key_iter: &mut Iter<'_, Key>,
    action: Action,
) {
    if !nodes.contains_key(key) {
        nodes.insert(key.clone(), Node::Key(HashMap::new()));
    }

    if let Some(Node::Key(hm)) = nodes.get_mut(key) {
        if let Some(next_key) = key_iter.next() {
            add_mapping_node(hm, next_key, key_iter, action)
        } else {
            nodes.insert(key.clone(), Node::Action(action));
        }
    }
}

fn get_node_from_tree(
    nodes: &HashMap<Key, Node>,
    key: &Key,
    key_iter: &mut Iter<'_, Key>,
) -> Option<Node> {
    if let Some(node) = nodes.get(key) {
        if let Some(next_key) = key_iter.next() {
            if let Node::Key(hm) = node {
                return get_node_from_tree(hm, next_key, key_iter);
            }
        }

        return Some(node.clone());
    }

    None
}
