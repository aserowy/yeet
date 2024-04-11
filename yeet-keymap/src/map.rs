use std::{collections::HashMap, path::PathBuf};

use regex::Regex;
use yeet_buffer::{
    message::{
        BufferMessage, CursorDirection, LineDirection, SearchDirection, TextModification,
        ViewPortDirection,
    },
    model::{CommandMode, Mode},
};

use crate::{
    key::{Key, KeyCode, KeyModifier},
    message::{Binding, BindingKind, Message, NextBindingKind},
    tree::KeyTree,
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
                tree.add_mapping(&mode, keys, message)
                    .expect("Default mappings must form a valid tree.");
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
            vec![
                Mode::Command(CommandMode::Command),
                Mode::Command(CommandMode::Search(SearchDirection::Up)),
                Mode::Command(CommandMode::Search(SearchDirection::Down)),
            ],
            vec![
                (
                    vec![Key::new(KeyCode::Backspace, vec![])],
                    Binding {
                        kind: BindingKind::Modification(TextModification::DeleteMotion(
                            1,
                            CursorDirection::Left,
                        )),
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
                        kind: BindingKind::Message(Message::LeaveCommandMode),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::Delete, vec![])],
                    Binding {
                        kind: BindingKind::Modification(TextModification::DeleteMotion(
                            1,
                            CursorDirection::Right,
                        )),
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
                    vec![
                        Key::new(KeyCode::from_char('"'), vec![]),
                        Key::new(KeyCode::from_char('p'), vec![]),
                    ],
                    Binding {
                        expects: Some(NextBindingKind::Raw(None)),
                        kind: BindingKind::Message(Message::PasteFromJunkYard(' ')),
                        ..Default::default()
                    },
                ),
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
                        kind: BindingKind::Repeat,
                        repeat: Some(0),
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
                        kind: BindingKind::Modification(TextModification::DeleteLine),
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
                    vec![
                        Key::new(KeyCode::from_char('g'), vec![]),
                        Key::new(KeyCode::from_char('n'), vec![]),
                    ],
                    Binding {
                        force: Some(Mode::Normal),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('p'), vec![])],
                    Binding {
                        kind: BindingKind::Message(Message::PasteFromJunkYard('"')),
                        ..Default::default()
                    },
                ),
                (
                    vec![
                        Key::new(KeyCode::from_char('y'), vec![]),
                        Key::new(KeyCode::from_char('y'), vec![]),
                    ],
                    Binding {
                        kind: BindingKind::Message(Message::YankToJunkYard(0)),
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
                    vec![Key::new(KeyCode::Space, vec![])],
                    Binding {
                        kind: BindingKind::Message(Message::ToggleQuickFix),
                        repeatable: false,
                        ..Default::default()
                    },
                ),
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
                        kind: BindingKind::Repeat,
                        repeat: Some(1),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('2'), vec![])],
                    Binding {
                        kind: BindingKind::Repeat,
                        repeat: Some(2),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('3'), vec![])],
                    Binding {
                        kind: BindingKind::Repeat,
                        repeat: Some(3),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('4'), vec![])],
                    Binding {
                        kind: BindingKind::Repeat,
                        repeat: Some(4),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('5'), vec![])],
                    Binding {
                        kind: BindingKind::Repeat,
                        repeat: Some(5),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('6'), vec![])],
                    Binding {
                        kind: BindingKind::Repeat,
                        repeat: Some(6),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('7'), vec![])],
                    Binding {
                        kind: BindingKind::Repeat,
                        repeat: Some(7),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('8'), vec![])],
                    Binding {
                        kind: BindingKind::Repeat,
                        repeat: Some(8),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('9'), vec![])],
                    Binding {
                        kind: BindingKind::Repeat,
                        repeat: Some(9),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('\''), vec![])],
                    Binding {
                        expects: Some(NextBindingKind::Raw(None)),
                        kind: BindingKind::Message(Message::NavigateToMark(' ')),
                        repeatable: false,
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char(':'), vec![])],
                    Binding {
                        force: Some(Mode::Command(CommandMode::Command)),
                        repeatable: false,
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('/'), vec![])],
                    Binding {
                        force: Some(Mode::Command(CommandMode::Search(SearchDirection::Down))),
                        repeatable: false,
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('?'), vec![])],
                    Binding {
                        force: Some(Mode::Command(CommandMode::Search(SearchDirection::Up))),
                        repeatable: false,
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('a'), vec![KeyModifier::Shift])],
                    Binding {
                        force: Some(Mode::Insert),
                        kind: BindingKind::Motion(CursorDirection::LineEnd),
                        repeat: None,
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('d'), vec![KeyModifier::Ctrl])],
                    Binding {
                        kind: BindingKind::Message(Message::Buffer(BufferMessage::MoveViewPort(
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
                    vec![Key::new(KeyCode::from_char('i'), vec![KeyModifier::Shift])],
                    Binding {
                        force: Some(Mode::Insert),
                        kind: BindingKind::Motion(CursorDirection::LineStart),
                        repeat: None,
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
                    vec![Key::new(KeyCode::from_char('m'), vec![])],
                    Binding {
                        expects: Some(NextBindingKind::Raw(Some(
                            Regex::new("[[:alpha:]]").expect("Invalid regex"),
                        ))),
                        kind: BindingKind::Message(Message::SetMark(' ')),
                        repeatable: false,
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('n'), vec![])],
                    Binding {
                        kind: BindingKind::Motion(CursorDirection::Search(true)),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('n'), vec![KeyModifier::Shift])],
                    Binding {
                        kind: BindingKind::Motion(CursorDirection::Search(false)),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('o'), vec![])],
                    Binding {
                        force: Some(Mode::Insert),
                        kind: BindingKind::Modification(TextModification::InsertNewLine(
                            LineDirection::Down,
                        )),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('o'), vec![KeyModifier::Shift])],
                    Binding {
                        force: Some(Mode::Insert),
                        kind: BindingKind::Modification(TextModification::InsertNewLine(
                            LineDirection::Up,
                        )),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('q'), vec![])],
                    Binding {
                        expects: Some(NextBindingKind::Raw(Some(
                            Regex::new("[[:alpha:]]").expect("Invalid regex"),
                        ))),
                        kind: BindingKind::Message(Message::StartMacro(' ')),
                        repeatable: false,
                        toggle: Some((
                            "macro-toggle".to_owned(),
                            BindingKind::Message(Message::StopMacro),
                        )),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('u'), vec![KeyModifier::Ctrl])],
                    Binding {
                        kind: BindingKind::Message(Message::Buffer(BufferMessage::MoveViewPort(
                            ViewPortDirection::HalfPageUp,
                        ))),
                        ..Default::default()
                    },
                ),
                (
                    vec![
                        Key::new(KeyCode::from_char('z'), vec![]),
                        Key::new(KeyCode::from_char('b'), vec![]),
                    ],
                    Binding {
                        kind: BindingKind::Message(Message::Buffer(BufferMessage::MoveViewPort(
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
                        kind: BindingKind::Message(Message::Buffer(BufferMessage::MoveViewPort(
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
                        kind: BindingKind::Message(Message::Buffer(BufferMessage::MoveViewPort(
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
                        kind: BindingKind::RepeatOrMotion(CursorDirection::LineStart),
                        repeat: Some(0),
                        repeatable: false,
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('.'), vec![])],
                    Binding {
                        kind: BindingKind::Message(Message::ExecuteRegister('.')),
                        ..Default::default()
                    },
                ),
                (
                    // TODO: add support for ,
                    vec![Key::new(KeyCode::from_char(';'), vec![])],
                    Binding {
                        kind: BindingKind::Message(Message::ExecuteRegister(';')),
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
                    vec![Key::new(KeyCode::from_char('a'), vec![])],
                    Binding {
                        force: Some(Mode::Insert),
                        kind: BindingKind::Motion(CursorDirection::Right),
                        repeat: None,
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('c'), vec![])],
                    Binding {
                        expects: Some(NextBindingKind::Motion),
                        force: Some(Mode::Insert),
                        kind: BindingKind::Modification(TextModification::DeleteMotion(
                            0,
                            CursorDirection::Right,
                        )),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('d'), vec![])],
                    Binding {
                        expects: Some(NextBindingKind::Motion),
                        kind: BindingKind::Modification(TextModification::DeleteMotion(
                            0,
                            CursorDirection::Right,
                        )),
                        ..Default::default()
                    },
                ),
                (
                    vec![
                        Key::new(KeyCode::from_char('d'), vec![]),
                        Key::new(KeyCode::from_char('d'), vec![]),
                    ],
                    Binding {
                        kind: BindingKind::Modification(TextModification::DeleteLine),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('f'), vec![])],
                    Binding {
                        expects: Some(NextBindingKind::Raw(None)),
                        kind: BindingKind::Motion(CursorDirection::FindForward('_')),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('f'), vec![KeyModifier::Shift])],
                    Binding {
                        expects: Some(NextBindingKind::Raw(None)),
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
                    vec![Key::new(KeyCode::from_char('i'), vec![])],
                    Binding {
                        force: Some(Mode::Insert),
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
                (
                    vec![Key::new(KeyCode::from_char('s'), vec![])],
                    Binding {
                        force: Some(Mode::Insert),
                        kind: BindingKind::Modification(TextModification::DeleteMotion(
                            1,
                            CursorDirection::Right,
                        )),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('t'), vec![])],
                    Binding {
                        expects: Some(NextBindingKind::Raw(None)),
                        kind: BindingKind::Motion(CursorDirection::TillForward('_')),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('t'), vec![KeyModifier::Shift])],
                    Binding {
                        expects: Some(NextBindingKind::Raw(None)),
                        kind: BindingKind::Motion(CursorDirection::TillBackward('_')),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::from_char('x'), vec![])],
                    Binding {
                        kind: BindingKind::Modification(TextModification::DeleteMotion(
                            1,
                            CursorDirection::Right,
                        )),
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
                        kind: BindingKind::Modification(TextModification::DeleteMotion(
                            1,
                            CursorDirection::Left,
                        )),
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
                        kind: BindingKind::Modification(TextModification::DeleteMotion(
                            1,
                            CursorDirection::Right,
                        )),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::Enter, vec![])],
                    Binding {
                        kind: BindingKind::Modification(TextModification::InsertLineBreak),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::Up, vec![])],
                    Binding {
                        kind: BindingKind::Motion(CursorDirection::Up),
                        ..Default::default()
                    },
                ),
                (
                    vec![Key::new(KeyCode::Down, vec![])],
                    Binding {
                        kind: BindingKind::Motion(CursorDirection::Down),
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
