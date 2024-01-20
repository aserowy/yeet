use std::collections::HashMap;

use crate::{
    action::Direction,
    key::{Key, KeyCode, KeyModifier},
    tree::KeyTree,
    Action, Mode,
};

#[derive(Debug)]
pub struct KeyMap {
    mappings: HashMap<Mode, Vec<(Vec<Key>, Action)>>,
}

impl KeyMap {
    pub fn into_tree(self) -> KeyTree {
        let mut tree = KeyTree::new();
        for (mode, mappings) in self.mappings {
            for (keys, action) in mappings {
                tree.add_mapping(&mode, keys, action);
            }
        }
        tree
    }
}

impl Default for KeyMap {
    fn default() -> Self {
        let mut mappings = HashMap::new();
        mappings.insert(
            Mode::Normal,
            vec![
                (
                    vec![Key::new(KeyCode::Esc, vec![])],
                    Action::ChangeMode(Mode::Normal),
                ),
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
                    // TODO: remove q and implement :q
                    vec![Key::new(KeyCode::from_char('q'), vec![])],
                    Action::Quit,
                ),
            ],
        );

        Self { mappings }
    }
}
