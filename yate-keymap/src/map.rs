use std::collections::HashMap;

use crate::{
    message::CursorDirection,
    key::{Key, KeyCode, KeyModifier},
    tree::KeyTree,
    Message, Mode,
};

#[derive(Debug)]
pub struct KeyMap {
    mappings: HashMap<Mode, Vec<(Vec<Key>, Message)>>,
}

impl KeyMap {
    pub fn into_tree(self) -> KeyTree {
        let mut tree = KeyTree::new();
        for (mode, mappings) in self.mappings {
            for (keys, message) in mappings {
                tree.add_mapping(&mode, keys, message);
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
                    Message::ChangeMode(Mode::Normal),
                ),
                (
                    vec![Key::new(KeyCode::from_char('0'), vec![])],
                    Message::MoveCursor(CursorDirection::LineStart),
                ),
                (
                    vec![Key::new(KeyCode::from_char('$'), vec![])],
                    Message::MoveCursor(CursorDirection::LineEnd),
                ),
                (
                    vec![Key::new(KeyCode::from_char('g'), vec![KeyModifier::Shift])],
                    Message::MoveCursor(CursorDirection::Bottom),
                ),
                (
                    vec![
                        Key::new(KeyCode::from_char('g'), vec![]),
                        Key::new(KeyCode::from_char('g'), vec![]),
                    ],
                    Message::MoveCursor(CursorDirection::Top),
                ),
                (
                    vec![Key::new(KeyCode::from_char('h'), vec![KeyModifier::Ctrl])],
                    Message::SelectParent,
                ),
                (
                    vec![Key::new(KeyCode::from_char('h'), vec![])],
                    Message::MoveCursor(CursorDirection::Left),
                ),
                (
                    vec![Key::new(KeyCode::from_char('j'), vec![])],
                    Message::MoveCursor(CursorDirection::Down),
                ),
                (
                    vec![Key::new(KeyCode::from_char('k'), vec![])],
                    Message::MoveCursor(CursorDirection::Up),
                ),
                (
                    vec![Key::new(KeyCode::from_char('l'), vec![KeyModifier::Ctrl])],
                    Message::SelectCurrent,
                ),
                (
                    vec![Key::new(KeyCode::from_char('l'), vec![])],
                    Message::MoveCursor(CursorDirection::Right),
                ),
                (
                    // TODO: remove q and implement :q
                    vec![Key::new(KeyCode::from_char('q'), vec![])],
                    Message::Quit,
                ),
            ],
        );

        Self { mappings }
    }
}
