use std::{collections::HashMap, slice::Iter};

use crate::{
    key::Key,
    message::{Binding, Mode},
    KeyMapError,
};

#[derive(Debug, Default)]
pub struct KeyTree {
    modes: HashMap<Mode, HashMap<Key, Node>>,
}

#[derive(Clone, Debug)]
pub enum Node {
    Key(HashMap<Key, Node>),
    Binding(Binding),
}

impl KeyTree {
    pub fn add_mapping(
        &mut self,
        mode: &Mode,
        keys: Vec<Key>,
        binding: Binding,
    ) -> Result<(), KeyMapError> {
        if !self.modes.contains_key(&mode) {
            self.modes.insert(mode.clone(), HashMap::new());
        }

        let mut key_iter = keys.iter();
        if let Some(key) = key_iter.next() {
            match self.modes.get_mut(&mode) {
                Some(mode) => {
                    add_mapping_node(mode, key, &mut key_iter, binding);

                    Ok(())
                }
                None => Err(KeyMapError::ModeUnresolvable(mode.to_string())),
            }
        } else {
            Ok(())
        }
    }

    pub fn get_bindings(&self, mode: &Mode, keys: &[Key]) -> (Vec<Binding>, Option<Node>) {
        if let Some(node) = self.modes.get(&mode) {
            let mut bindings = Vec::new();

            let mut key_iter = keys.iter();
            while let Some(key) = key_iter.next() {
                match get_node_from_tree(node, key, &mut key_iter) {
                    Some(node) => match node {
                        Node::Key(_) => return (bindings, Some(node)),
                        Node::Binding(binding) => bindings.push(binding),
                    },
                    None => return (vec![], None),
                }
            }

            (bindings, None)
        } else {
            (vec![], None)
        }
    }
}

fn add_mapping_node(
    nodes: &mut HashMap<Key, Node>,
    key: &Key,
    key_iter: &mut Iter<'_, Key>,
    binding: Binding,
) {
    if !nodes.contains_key(key) {
        nodes.insert(key.clone(), Node::Key(HashMap::new()));
    }

    if let Some(Node::Key(hm)) = nodes.get_mut(key) {
        if let Some(next_key) = key_iter.next() {
            add_mapping_node(hm, next_key, key_iter, binding)
        } else {
            nodes.insert(key.clone(), Node::Binding(binding));
        }
    }
}

fn get_node_from_tree(
    nodes: &HashMap<Key, Node>,
    key: &Key,
    key_iter: &mut Iter<'_, Key>,
) -> Option<Node> {
    if let Some(node) = nodes.get(key) {
        if let Node::Binding(_) = node {
            return Some(node.clone());
        }

        if let Some(next_key) = key_iter.next() {
            if let Node::Key(hm) = node {
                return get_node_from_tree(hm, next_key, key_iter);
            }
        }

        Some(node.clone())
    } else {
        None
    }
}
