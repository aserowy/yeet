use std::collections::HashMap;

use crate::{
    action::Direction,
    key::{Key, KeyCode, KeyModifier},
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
            vec![
                (
                    vec![Key::new(KeyCode::from_char('0'), vec![])],
                    Action::MoveCursor(Direction::LineStart),
                ),
                (
                    vec![Key::new(KeyCode::from_char('$'), vec![])],
                    Action::MoveCursor(Direction::LineEnd),
                ),
                (
                    vec![Key::new(KeyCode::from_char('g'), vec![KeyModifier::Shift])],
                    Action::MoveCursor(Direction::Bottom),
                ),
                (
                    vec![
                        Key::new(KeyCode::from_char('g'), vec![]),
                        Key::new(KeyCode::from_char('g'), vec![]),
                    ],
                    Action::MoveCursor(Direction::Top),
                ),
                (
                    vec![Key::new(KeyCode::from_char('h'), vec![])],
                    Action::MoveCursor(Direction::Left),
                ),
                (
                    vec![Key::new(KeyCode::from_char('j'), vec![])],
                    Action::MoveCursor(Direction::Down),
                ),
                (
                    vec![Key::new(KeyCode::from_char('k'), vec![])],
                    Action::MoveCursor(Direction::Up),
                ),
                (
                    vec![Key::new(KeyCode::from_char('l'), vec![])],
                    Action::MoveCursor(Direction::Right),
                ),
                (
                    vec![Key::new(KeyCode::from_char('q'), vec![])],
                    Action::Quit,
                ),
            ],
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
