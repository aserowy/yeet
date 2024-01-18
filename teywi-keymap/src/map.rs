use std::collections::HashMap;

use crate::{
    key::{KeyCode, KeyModifier, Key},
    Action, Mode,
};

#[derive(Debug)]
pub struct KeyMap {
    mappings: HashMap<Mode, Vec<(Vec<Key>, Action)>>,
}

impl KeyMap {
    pub fn get_action(&self, mode: &Mode, keys: &Vec<Key>) -> Option<Action> {
        if let Some(mappings) = self.mappings.get(mode) {
            for (mapping_keys, action) in mappings {
                if mapping_keys.len() == keys.len() {
                    if compare_keys(mapping_keys, keys) {
                        return Some(action.clone());
                    }
                }
            }
        }

        None
    }
}

impl Default for KeyMap {
    fn default() -> Self {
        let mut mappings = HashMap::new();
        mappings.insert(
            Mode::Normal,
            vec![(
                vec![Key::new(KeyCode::Char('q'), vec![KeyModifier::Alt])],
                Action::Quit,
            )],
        );

        Self { mappings }
    }
}

fn compare_keys(a: &Vec<Key>, b: &Vec<Key>) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let zipped = a.iter().zip(b.iter());
    for (a, b) in zipped {
        if a.code != b.code {
            return false;
        }
        if !compare_modifiers(&a.modifiers, &b.modifiers) {
            return false;
        }
    }

    true
}

fn compare_modifiers(a: &Vec<KeyModifier>, b: &Vec<KeyModifier>) -> bool {
    if a.len() != b.len() {
        return false;
    }

    for modifier in a {
        if !b.contains(modifier) {
            return false;
        }
    }

    true
}
