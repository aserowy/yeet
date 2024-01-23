use std::collections::HashMap;

use crate::{
    key::{Key, KeyCode, KeyModifier},
    message::{Binding, CursorDirection, ViewPortDirection},
    tree::KeyTree,
    Message, Mode,
};

#[derive(Debug)]
pub struct KeyMap {
    mappings: HashMap<Mode, Vec<(Vec<Key>, Binding)>>,
}

impl KeyMap {
    pub fn into_tree(self) -> KeyTree {
        let mut tree = KeyTree::default();
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
                // TODO: delete (for testing only)
                (
                    vec![Key::new(KeyCode::Enter, vec![])],
                    Binding::Message(Message::SelectCurrent),
                ),
                (
                    vec![Key::new(KeyCode::Esc, vec![])],
                    Binding::Message(Message::ChangeMode(Mode::Normal)),
                ),
                (
                    vec![Key::new(KeyCode::from_char('0'), vec![])],
                    Binding::Motion(CursorDirection::LineStart),
                ),
                (
                    vec![Key::new(KeyCode::from_char('1'), vec![])],
                    Binding::Repeat(1),
                ),
                (
                    vec![Key::new(KeyCode::from_char('2'), vec![])],
                    Binding::Repeat(2),
                ),
                (
                    vec![Key::new(KeyCode::from_char('3'), vec![])],
                    Binding::Repeat(3),
                ),
                (
                    vec![Key::new(KeyCode::from_char('4'), vec![])],
                    Binding::Repeat(4),
                ),
                (
                    vec![Key::new(KeyCode::from_char('5'), vec![])],
                    Binding::Repeat(5),
                ),
                (
                    vec![Key::new(KeyCode::from_char('6'), vec![])],
                    Binding::Repeat(6),
                ),
                (
                    vec![Key::new(KeyCode::from_char('7'), vec![])],
                    Binding::Repeat(7),
                ),
                (
                    vec![Key::new(KeyCode::from_char('8'), vec![])],
                    Binding::Repeat(8),
                ),
                (
                    vec![Key::new(KeyCode::from_char('9'), vec![])],
                    Binding::Repeat(9),
                ),
                (
                    vec![Key::new(KeyCode::from_char('$'), vec![])],
                    Binding::Motion(CursorDirection::LineEnd),
                ),
                (
                    vec![Key::new(KeyCode::from_char('d'), vec![KeyModifier::Ctrl])],
                    Binding::Message(Message::MoveViewPort(ViewPortDirection::HalfPageDown)),
                ),
                (
                    vec![Key::new(KeyCode::from_char('g'), vec![KeyModifier::Shift])],
                    Binding::Motion(CursorDirection::Bottom),
                ),
                (
                    vec![
                        Key::new(KeyCode::from_char('g'), vec![]),
                        Key::new(KeyCode::from_char('g'), vec![]),
                    ],
                    Binding::Motion(CursorDirection::Top),
                ),
                (
                    vec![Key::new(KeyCode::from_char('h'), vec![KeyModifier::Shift])],
                    Binding::Message(Message::SelectParent),
                ),
                (
                    vec![Key::new(KeyCode::from_char('h'), vec![])],
                    Binding::Motion(CursorDirection::Left),
                ),
                (
                    vec![Key::new(KeyCode::from_char('j'), vec![])],
                    Binding::Motion(CursorDirection::Down),
                ),
                (
                    vec![Key::new(KeyCode::from_char('k'), vec![])],
                    Binding::Motion(CursorDirection::Up),
                ),
                (
                    vec![Key::new(KeyCode::from_char('l'), vec![KeyModifier::Shift])],
                    Binding::Message(Message::SelectCurrent),
                ),
                (
                    vec![Key::new(KeyCode::from_char('l'), vec![])],
                    Binding::Motion(CursorDirection::Right),
                ),
                (
                    vec![Key::new(KeyCode::from_char('u'), vec![KeyModifier::Ctrl])],
                    Binding::Message(Message::MoveViewPort(ViewPortDirection::HalfPageUp)),
                ),
                (
                    vec![
                        Key::new(KeyCode::from_char('z'), vec![]),
                        Key::new(KeyCode::from_char('b'), vec![]),
                    ],
                    Binding::Message(Message::MoveViewPort(ViewPortDirection::BottomOnCursor)),
                ),
                (
                    vec![
                        Key::new(KeyCode::from_char('z'), vec![]),
                        Key::new(KeyCode::from_char('t'), vec![]),
                    ],
                    Binding::Message(Message::MoveViewPort(ViewPortDirection::TopOnCursor)),
                ),
                (
                    vec![
                        Key::new(KeyCode::from_char('z'), vec![]),
                        Key::new(KeyCode::from_char('z'), vec![]),
                    ],
                    Binding::Message(Message::MoveViewPort(ViewPortDirection::CenterOnCursor)),
                ),
                (
                    // TODO: remove q and implement :q
                    vec![Key::new(KeyCode::from_char('q'), vec![])],
                    Binding::Message(Message::Quit),
                ),
            ],
        );

        Self { mappings }
    }
}
