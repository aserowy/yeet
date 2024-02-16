use std::collections::HashMap;

use crate::{
    key::{Key, KeyCode, KeyModifier},
    message::{
        Binding, Buffer, CursorDirection, NewLineDirection, TextModification, ViewPortDirection,
    },
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
                if let Err(_error) = tree.add_mapping(&mode, keys, message) {
                    // TODO: add logging
                }
            }
        }
        tree
    }
}

impl Default for KeyMap {
    fn default() -> Self {
        let mut mappings = HashMap::new();

        add_mapping(
            &mut mappings,
            vec![Mode::Command],
            vec![
                (
                    vec![Key::new(KeyCode::Backspace, vec![])],
                    Binding::Message(Message::Buffer(Buffer::Modification(
                        TextModification::DeleteCharBeforeCursor,
                    ))),
                ),
                (
                    vec![Key::new(KeyCode::Enter, vec![])],
                    Binding::Message(Message::ExecuteCommand),
                ),
                (
                    vec![Key::new(KeyCode::Esc, vec![])],
                    Binding::Mode(Mode::default()),
                ),
                (
                    vec![Key::new(KeyCode::Delete, vec![])],
                    Binding::Message(Message::Buffer(Buffer::Modification(
                        TextModification::DeleteCharOnCursor,
                    ))),
                ),
                (
                    vec![Key::new(KeyCode::Left, vec![])],
                    Binding::Motion(CursorDirection::Left),
                ),
                (
                    vec![Key::new(KeyCode::Right, vec![])],
                    Binding::Motion(CursorDirection::Right),
                ),
            ],
        );

        add_mapping(
            &mut mappings,
            vec![Mode::Insert],
            vec![
                (
                    vec![Key::new(KeyCode::Backspace, vec![])],
                    Binding::Message(Message::Buffer(Buffer::Modification(
                        TextModification::DeleteCharBeforeCursor,
                    ))),
                ),
                (
                    vec![Key::new(KeyCode::Enter, vec![])],
                    Binding::Message(Message::ExecuteCommand),
                ),
                (
                    vec![Key::new(KeyCode::Esc, vec![])],
                    Binding::Mode(Mode::Normal),
                ),
                (
                    vec![Key::new(KeyCode::Delete, vec![])],
                    Binding::Message(Message::Buffer(Buffer::Modification(
                        TextModification::DeleteCharOnCursor,
                    ))),
                ),
                (
                    vec![Key::new(KeyCode::Left, vec![])],
                    Binding::Motion(CursorDirection::Left),
                ),
                (
                    vec![Key::new(KeyCode::Right, vec![])],
                    Binding::Motion(CursorDirection::Right),
                ),
            ],
        );

        add_mapping(
            &mut mappings,
            vec![Mode::Navigation],
            vec![
                (
                    vec![Key::new(KeyCode::from_char('0'), vec![])],
                    Binding::Repeat(0),
                ),
                (
                    vec![
                        Key::new(KeyCode::from_char('d'), vec![]),
                        Key::new(KeyCode::from_char('d'), vec![]),
                    ],
                    Binding::ModeAndTextModification(
                        Mode::Normal,
                        TextModification::DeleteLineOnCursor,
                    ),
                ),
                (
                    vec![Key::new(KeyCode::from_char('h'), vec![])],
                    Binding::Message(Message::SelectParent),
                ),
                (
                    vec![Key::new(KeyCode::from_char('l'), vec![])],
                    Binding::Message(Message::SelectCurrent),
                ),
                (
                    vec![Key::new(KeyCode::from_char('m'), vec![])],
                    Binding::Mode(Mode::Normal),
                ),
            ],
        );

        add_mapping(
            &mut mappings,
            vec![Mode::Navigation, Mode::Normal],
            vec![
                (
                    vec![Key::new(KeyCode::Esc, vec![])],
                    Binding::Mode(Mode::Navigation),
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
                    vec![Key::new(KeyCode::from_char(':'), vec![])],
                    Binding::Mode(Mode::Command),
                ),
                (
                    vec![Key::new(KeyCode::from_char('a'), vec![])],
                    Binding::ModeAndNotRepeatedMotion(Mode::Insert, CursorDirection::Right),
                ),
                (
                    vec![Key::new(KeyCode::from_char('a'), vec![KeyModifier::Shift])],
                    Binding::ModeAndNotRepeatedMotion(Mode::Insert, CursorDirection::LineEnd),
                ),
                (
                    vec![Key::new(KeyCode::from_char('d'), vec![KeyModifier::Ctrl])],
                    Binding::Message(Message::Buffer(Buffer::MoveViewPort(
                        ViewPortDirection::HalfPageDown,
                    ))),
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
                    vec![Key::new(KeyCode::from_char('i'), vec![])],
                    Binding::Mode(Mode::Insert),
                ),
                (
                    vec![Key::new(KeyCode::from_char('i'), vec![KeyModifier::Shift])],
                    Binding::ModeAndNotRepeatedMotion(Mode::Insert, CursorDirection::LineStart),
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
                    vec![Key::new(KeyCode::from_char('o'), vec![])],
                    Binding::ModeAndTextModification(
                        Mode::Insert,
                        TextModification::InsertNewLine(NewLineDirection::Under),
                    ),
                ),
                (
                    vec![Key::new(KeyCode::from_char('o'), vec![KeyModifier::Shift])],
                    Binding::ModeAndTextModification(
                        Mode::Insert,
                        TextModification::InsertNewLine(NewLineDirection::Above),
                    ),
                ),
                (
                    vec![Key::new(KeyCode::from_char('u'), vec![KeyModifier::Ctrl])],
                    Binding::Message(Message::Buffer(Buffer::MoveViewPort(
                        ViewPortDirection::HalfPageUp,
                    ))),
                ),
                (
                    vec![
                        Key::new(KeyCode::from_char('z'), vec![]),
                        Key::new(KeyCode::from_char('b'), vec![]),
                    ],
                    Binding::Message(Message::Buffer(Buffer::MoveViewPort(
                        ViewPortDirection::BottomOnCursor,
                    ))),
                ),
                (
                    vec![
                        Key::new(KeyCode::from_char('z'), vec![]),
                        Key::new(KeyCode::from_char('t'), vec![]),
                    ],
                    Binding::Message(Message::Buffer(Buffer::MoveViewPort(
                        ViewPortDirection::TopOnCursor,
                    ))),
                ),
                (
                    vec![
                        Key::new(KeyCode::from_char('z'), vec![]),
                        Key::new(KeyCode::from_char('z'), vec![]),
                    ],
                    Binding::Message(Message::Buffer(Buffer::MoveViewPort(
                        ViewPortDirection::CenterOnCursor,
                    ))),
                ),
            ],
        );
        add_mapping(
            &mut mappings,
            vec![Mode::Normal],
            vec![
                (
                    vec![Key::new(KeyCode::from_char('0'), vec![])],
                    Binding::RepeatOrMotion(0, CursorDirection::LineStart),
                ),
                (
                    vec![Key::new(KeyCode::from_char('$'), vec![])],
                    Binding::Motion(CursorDirection::LineEnd),
                ),
                (
                    vec![
                        Key::new(KeyCode::from_char('d'), vec![]),
                        Key::new(KeyCode::from_char('d'), vec![]),
                    ],
                    Binding::Message(Message::Buffer(Buffer::Modification(
                        TextModification::DeleteLineOnCursor,
                    ))),
                ),
                (
                    vec![Key::new(KeyCode::from_char('h'), vec![])],
                    Binding::Motion(CursorDirection::Left),
                ),
                (
                    vec![Key::new(KeyCode::from_char('l'), vec![])],
                    Binding::Motion(CursorDirection::Right),
                ),
            ],
        );

        Self { mappings }
    }
}

fn add_mapping(
    mappings: &mut HashMap<Mode, Vec<(Vec<Key>, Binding)>>,
    modes: Vec<Mode>,
    bindings: Vec<(Vec<Key>, Binding)>,
) {
    for mode in modes {
        if let Some(mappings_for_mode) = mappings.get_mut(&mode) {
            mappings_for_mode.extend(bindings.clone());
        } else {
            mappings.insert(mode, bindings.clone());
        }
    }
}
