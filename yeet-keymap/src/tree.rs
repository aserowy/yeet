use std::{collections::HashMap, iter::Enumerate, slice::Iter};

use crate::{
    key::Key,
    message::{Binding, BindingKind, Mode, NextBindingKind},
    KeyMapError,
};

#[derive(Debug, Default)]
pub struct KeyTree {
    modes: HashMap<Mode, Node>,
}

#[derive(Clone, Debug)]
pub enum Node {
    Binding(Binding),
    ExpectsOr(Binding, HashMap<Key, Node>),
    Key(HashMap<Key, Node>),
}

impl KeyTree {
    pub fn add_mapping(
        &mut self,
        mode: &Mode,
        keys: Vec<Key>,
        binding: Binding,
    ) -> Result<(), KeyMapError> {
        if !self.modes.contains_key(mode) {
            self.modes.insert(mode.clone(), Node::Key(HashMap::new()));
        }

        let max_index = keys.len() - 1;
        let mut iter = keys.iter().enumerate();
        match self.modes.get_mut(mode) {
            Some(node) => {
                add_mapping_node(&max_index, &mut iter, node, binding);
                Ok(())
            }
            None => Err(KeyMapError::ModeUnresolvable(mode.to_string())),
        }
    }

    pub fn get_bindings(
        &self,
        mode: &Mode,
        keys: &[Key],
    ) -> Result<Option<Vec<Binding>>, KeyMapError> {
        if let Some(node) = self.modes.get(mode) {
            let mut key_iter = keys.iter();

            let mut bindings = Vec::new();
            while let Some(node) = get_bindings_from_node(node, true, &mut key_iter)? {
                let expects = match node {
                    Node::Binding(binding) => {
                        bindings.push(binding.clone());
                        binding.expects
                    }
                    Node::ExpectsOr(binding, _) => {
                        bindings.push(binding.clone());
                        binding.expects
                    }
                    Node::Key(_) => return Ok(None),
                };

                if let Some(NextBindingKind::Raw) = expects {
                    let key = match key_iter.next() {
                        Some(it) => it,
                        None => return Ok(None),
                    };

                    let string = key.to_string();
                    let chars: Vec<_> = string.chars().collect();
                    if chars.len() != 1 {
                        return Err(KeyMapError::NoValidBindingFound);
                    }

                    bindings.push(Binding {
                        kind: BindingKind::Raw(chars[0]),
                        ..Default::default()
                    });
                }
            }

            Ok(Some(bindings))
        } else {
            Err(KeyMapError::ModeUnresolvable(mode.to_string()))
        }
    }
}

fn add_mapping_node(
    max_index: &usize,
    iter: &mut Enumerate<Iter<'_, Key>>,
    node: &mut Node,
    binding: Binding,
) {
    if let Some((index, key)) = iter.next() {
        if &index == max_index {
            match node {
                Node::Binding(_) => unreachable!(),
                Node::ExpectsOr(_, map) | Node::Key(map) => {
                    if binding.expects.is_some() {
                        if let Some(node) = map.remove(key) {
                            match node {
                                Node::Binding(_) | Node::ExpectsOr(_, _) => unreachable!(),
                                Node::Key(m) => {
                                    map.insert(key.clone(), Node::ExpectsOr(binding, m.clone()));
                                }
                            }
                        } else {
                            map.insert(key.clone(), Node::ExpectsOr(binding, HashMap::new()));
                        }
                    } else if map.insert(key.clone(), Node::Binding(binding)).is_some() {
                        panic!("This should not happen");
                    }
                }
            }
        } else {
            match node {
                Node::Binding(_) => unreachable!(),
                Node::ExpectsOr(_, map) | Node::Key(map) => {
                    if !map.contains_key(key) {
                        map.insert(key.clone(), Node::Key(HashMap::new()));
                    }
                    let node = map.get_mut(key).expect("Must exist");
                    add_mapping_node(max_index, iter, node, binding);
                }
            }
        }
    }
}

fn get_bindings_from_node(
    node: &Node,
    initial_node: bool,
    iter: &mut Iter<'_, Key>,
) -> Result<Option<Node>, KeyMapError> {
    match node {
        Node::Binding(_) => Ok(Some(node.clone())),
        Node::ExpectsOr(_, map) | Node::Key(map) => {
            let mut peak_iter = iter.clone();
            let key = match peak_iter.next() {
                Some(it) => it,
                None => {
                    if initial_node {
                        return Ok(None);
                    } else {
                        return Ok(Some(node.clone()));
                    }
                }
            };

            if let Some(node) = map.get(key) {
                let _ = iter.next();
                get_bindings_from_node(node, false, iter)
            } else {
                match node {
                    Node::ExpectsOr(_, _) => Ok(Some(node.clone())),
                    Node::Key(_) => Err(KeyMapError::NoValidBindingFound),
                    Node::Binding(_) => unreachable!(),
                }
            }
        }
    }
}
