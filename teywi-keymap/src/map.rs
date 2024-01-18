use std::collections::HashMap;

use crate::{
    key::{KeyCode, KeyModifier},
    Action, Mode,
};

#[derive(Clone, Debug, PartialEq)]
pub struct KeyStroke {
    pub key: KeyCode,
    pub modifiers: Vec<KeyModifier>,
}

pub struct KeyMap {
    mappings: HashMap<Mode, Vec<(Vec<KeyStroke>, Action)>>,
}

impl KeyMap {
    pub fn get_action(&self, mode: &Mode, keystrokes: &Vec<KeyStroke>) -> Option<Action> {
        if let Some(mappings) = self.mappings.get(mode) {
            for (mapping_keystrokes, action) in mappings {
                if mapping_keystrokes.len() == keystrokes.len() {
                    if compare_keystrokes(mapping_keystrokes, keystrokes) {
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
        let mappings = HashMap::new();

        Self {
            mappings,
        }
    }
}

fn compare_keystrokes(a: &Vec<KeyStroke>, b: &Vec<KeyStroke>) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let zipped = a.iter().zip(b.iter());
    for (a, b) in zipped {
        if a.key != b.key {
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
