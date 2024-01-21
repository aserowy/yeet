use std::collections::HashMap;

use crate::{
    action::CursorDirection,
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
                    Action::MoveCursor(CursorDirection::LineStart),
                ),
                (
                    vec![Key::new(KeyCode::from_char('$'), vec![])],
                    Action::MoveCursor(CursorDirection::LineEnd),
                ),
                (
                    vec![Key::new(KeyCode::from_char('g'), vec![KeyModifier::Shift])],
                    Action::MoveCursor(CursorDirection::Bottom),
                ),
                (
                    vec![
                        Key::new(KeyCode::from_char('g'), vec![]),
                        Key::new(KeyCode::from_char('g'), vec![]),
                    ],
                    Action::MoveCursor(CursorDirection::Top),
                ),
                (
                    vec![Key::new(KeyCode::from_char('h'), vec![])],
                    Action::MoveCursor(CursorDirection::Left),
                ),
                (
                    vec![Key::new(KeyCode::from_char('j'), vec![])],
                    Action::MoveCursor(CursorDirection::Down),
                ),
                (
                    vec![Key::new(KeyCode::from_char('k'), vec![])],
                    Action::MoveCursor(CursorDirection::Up),
                ),
                (
                    vec![Key::new(KeyCode::from_char('l'), vec![])],
                    Action::MoveCursor(CursorDirection::Right),
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
