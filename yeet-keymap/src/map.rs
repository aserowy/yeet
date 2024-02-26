use std::{collections::HashMap, path::PathBuf};

use crate::{
    key::{Key, KeyCode, KeyModifier},
    message::{
        Binding, BindingKind, Buffer, CursorDirection, NewLineDirection, NextBindingKind,
        TextModification, ViewPortDirection,
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
    #[allow(clippy::needless_update)]
    fn default() -> Self {
        let mut mappings = HashMap::new();

        add_mapping(
            &mut mappings,
            vec![Mode::Command],
            vec![
                (
                    vec![Key::new(KeyCode::Backspace, vec![])],
                    Binding {
                        kind: BindingKind::Modification(TextModification::DeleteCharBeforeCursor),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::Enter, vec![])],
                    Binding {
                        kind: BindingKind::Message(Message::ExecuteCommand),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::Esc, vec![])],
                    Binding {
                        force: Some(Mode::default()),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::Delete, vec![])],
                    Binding {
                        kind: BindingKind::Modification(TextModification::DeleteCharOnCursor),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::Left, vec![])],
                    Binding {
                        kind: BindingKind::Motion(CursorDirection::Left),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::Right, vec![])],
                    Binding {
                        kind: BindingKind::Motion(CursorDirection::Right),
                        ..Default::default()
                    },
                ),
            ],
        );

        add_mapping(
            &mut mappings,
            vec![Mode::Navigation],
            vec![
                (
                    vec![Key::new(KeyCode::Enter, vec![])],
                    Binding {
                        kind: BindingKind::Message(Message::OpenSelected),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('0'), vec![])],
                    Binding {
                        kind: BindingKind::Repeat(0),
                        ..Default::default()
                    },
                ),
                (
                    vec![
                        Key::new(KeyCode::from_char('d'), vec![]),
                        Key::new(KeyCode::from_char('d'), vec![]),
                    ],
                    Binding {
                        force: Some(Mode::Normal),
                        kind: BindingKind::Modification(TextModification::DeleteLineOnCursor),
                        ..Default::default()
                    },
                ),
                (
                    vec![
                        Key::new(KeyCode::from_char('g'), vec![]),
                        Key::new(KeyCode::from_char('h'), vec![]),
                    ],
                    Binding {
                        kind: BindingKind::Message(Message::NavigateToPath(get_home_path())),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('h'), vec![])],
                    Binding {
                        kind: BindingKind::Message(Message::NavigateToParent),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('l'), vec![])],
                    Binding {
                        kind: BindingKind::Message(Message::NavigateToSelected),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('m'), vec![])],
                    Binding {
                        force: Some(Mode::Normal),
                        ..Default::default()
                    },
                ),
            ],
        );

        add_mapping(
            &mut mappings,
            vec![Mode::Navigation, Mode::Normal],
            vec![
                (
                    vec![Key::new(KeyCode::Esc, vec![])],
                    Binding {
                        force: Some(Mode::Navigation),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('1'), vec![])],
                    Binding {
                        kind: BindingKind::Repeat(1),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('2'), vec![])],
                    Binding {
                        kind: BindingKind::Repeat(2),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('3'), vec![])],
                    Binding {
                        kind: BindingKind::Repeat(3),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('4'), vec![])],
                    Binding {
                        kind: BindingKind::Repeat(4),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('5'), vec![])],
                    Binding {
                        kind: BindingKind::Repeat(5),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('6'), vec![])],
                    Binding {
                        kind: BindingKind::Repeat(6),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('7'), vec![])],
                    Binding {
                        kind: BindingKind::Repeat(7),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('8'), vec![])],
                    Binding {
                        kind: BindingKind::Repeat(8),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('9'), vec![])],
                    Binding {
                        kind: BindingKind::Repeat(9),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char(':'), vec![])],
                    Binding {
                        force: Some(Mode::Command),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('a'), vec![])],
                    Binding {
                        force: Some(Mode::Insert),
                        kind: BindingKind::Motion(CursorDirection::Right),
                        repeatable: false,
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('a'), vec![KeyModifier::Shift])],
                    Binding {
                        force: Some(Mode::Insert),
                        kind: BindingKind::Motion(CursorDirection::LineEnd),
                        repeatable: false,
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('d'), vec![KeyModifier::Ctrl])],
                    Binding {
                        kind: BindingKind::Message(Message::Buffer(Buffer::MoveViewPort(
                            ViewPortDirection::HalfPageDown,
                        ))),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('g'), vec![KeyModifier::Shift])],
                    Binding {
                        kind: BindingKind::Motion(CursorDirection::Bottom),
                        ..Default::default()
                    },
                ),
                (
                    vec![
                        Key::new(KeyCode::from_char('g'), vec![]),
                        Key::new(KeyCode::from_char('g'), vec![]),
                    ],
                    Binding {
                        kind: BindingKind::Motion(CursorDirection::Top),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('i'), vec![])],
                    Binding {
                        force: Some(Mode::Insert),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('i'), vec![KeyModifier::Shift])],
                    Binding {
                        force: Some(Mode::Insert),
                        kind: BindingKind::Motion(CursorDirection::LineStart),
                        repeatable: false,
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('j'), vec![])],
                    Binding {
                        kind: BindingKind::Motion(CursorDirection::Down),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('k'), vec![])],
                    Binding {
                        kind: BindingKind::Motion(CursorDirection::Up),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('o'), vec![])],
                    Binding {
                        force: Some(Mode::Insert),
                        kind: BindingKind::Modification(TextModification::InsertNewLine(
                            NewLineDirection::Under,
                        )),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('o'), vec![KeyModifier::Shift])],
                    Binding {
                        force: Some(Mode::Insert),
                        kind: BindingKind::Modification(TextModification::InsertNewLine(
                            NewLineDirection::Above,
                        )),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('p'), vec![])],
                    Binding {
                        kind: BindingKind::Message(Message::PasteRegister("\"".to_string())),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('u'), vec![KeyModifier::Ctrl])],
                    Binding {
                        kind: BindingKind::Message(Message::Buffer(Buffer::MoveViewPort(
                            ViewPortDirection::HalfPageUp,
                        ))),
                        ..Default::default()
                    },
                ),
                (
                    vec![
                        Key::new(KeyCode::from_char('y'), vec![]),
                        Key::new(KeyCode::from_char('y'), vec![]),
                    ],
                    Binding {
                        kind: BindingKind::Message(Message::YankSelected(1)),
                        ..Default::default()
                    },
                ),
                (
                    vec![
                        Key::new(KeyCode::from_char('z'), vec![]),
                        Key::new(KeyCode::from_char('b'), vec![]),
                    ],
                    Binding {
                        kind: BindingKind::Message(Message::Buffer(Buffer::MoveViewPort(
                            ViewPortDirection::BottomOnCursor,
                        ))),
                        ..Default::default()
                    },
                ),
                (
                    vec![
                        Key::new(KeyCode::from_char('z'), vec![]),
                        Key::new(KeyCode::from_char('t'), vec![]),
                    ],
                    Binding {
                        kind: BindingKind::Message(Message::Buffer(Buffer::MoveViewPort(
                            ViewPortDirection::TopOnCursor,
                        ))),
                        ..Default::default()
                    },
                ),
                (
                    vec![
                        Key::new(KeyCode::from_char('z'), vec![]),
                        Key::new(KeyCode::from_char('z'), vec![]),
                    ],
                    Binding {
                        kind: BindingKind::Message(Message::Buffer(Buffer::MoveViewPort(
                            ViewPortDirection::CenterOnCursor,
                        ))),
                        ..Default::default()
                    },
                ),
            ],
        );

        add_mapping(
            &mut mappings,
            vec![Mode::Normal],
            vec![
                (
                    vec![Key::new(KeyCode::from_char('0'), vec![])],
                    Binding {
                        kind: BindingKind::RepeatOrMotion(0, CursorDirection::LineStart),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('$'), vec![])],
                    Binding {
                        kind: BindingKind::Motion(CursorDirection::LineEnd),
                        ..Default::default()
                    },
                ),
                (
                    vec![
                        Key::new(KeyCode::from_char('d'), vec![]),
                        Key::new(KeyCode::from_char('d'), vec![]),
                    ],
                    Binding {
                        kind: BindingKind::Modification(TextModification::DeleteLineOnCursor),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('f'), vec![])],
                    Binding {
                        expects: Some(NextBindingKind::Raw),
                        kind: BindingKind::Motion(CursorDirection::FindForward('_')),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('f'), vec![KeyModifier::Shift])],
                    Binding {
                        expects: Some(NextBindingKind::Raw),
                        kind: BindingKind::Motion(CursorDirection::FindBackward('_')),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('h'), vec![])],
                    Binding {
                        kind: BindingKind::Motion(CursorDirection::Left),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('l'), vec![])],
                    Binding {
                        kind: BindingKind::Motion(CursorDirection::Right),
                        ..Default::default()
                    },
                ),
            ],
        );

        add_mapping(
            &mut mappings,
            vec![Mode::Insert],
            vec![
                (
                    vec![Key::new(KeyCode::Backspace, vec![])],
                    Binding {
                        kind: BindingKind::Modification(TextModification::DeleteCharBeforeCursor),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::Esc, vec![])],
                    Binding {
                        force: Some(Mode::Normal),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::Delete, vec![])],
                    Binding {
                        kind: BindingKind::Modification(TextModification::DeleteCharOnCursor),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::Left, vec![])],
                    Binding {
                        kind: BindingKind::Motion(CursorDirection::Left),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::Right, vec![])],
                    Binding {
                        kind: BindingKind::Motion(CursorDirection::Right),
                        ..Default::default()
                    },
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

fn get_home_path() -> PathBuf {
    dirs::home_dir()
        .expect("Failed to get home directory")
        .to_path_buf()
}
