use yeet_buffer::{
    message::{BufferMessage, CursorDirection, TextModification},
    model::{CommandMode, Mode},
};
use yeet_keymap::{
    key::{Key, KeyCode, KeyModifier},
    message::{KeySequence, KeymapMessage},
    MessageResolver,
};

#[test]
fn add_and_resolve_key_navigation_colon() {
    let mut resolver = MessageResolver::default();
    let result = resolver.add_key(Key::new(KeyCode::from_char(':'), vec![]));

    println!("{:?}", result);

    assert_eq!(
        Some(&KeymapMessage::Buffer(BufferMessage::ChangeMode(
            Mode::Navigation,
            Mode::Command(CommandMode::Command)
        ))),
        result.0.first()
    );
    assert_eq!(KeySequence::Completed(":".to_string()), result.1);
    assert_eq!(1, result.0.len());
}

#[test]
fn add_and_resolve_key_navigation_d() {
    let mut resolver = MessageResolver::default();
    let result = resolver.add_key(Key::new(KeyCode::from_char('d'), vec![]));

    println!("{:?}", result);

    assert_eq!(KeySequence::Changed("d".to_string()), result.1);
    assert!(result.0.is_empty());
}

#[test]
fn add_and_resolve_key_navigation_dq() {
    let mut resolver = MessageResolver::default();
    let _ = resolver.add_key(Key::new(KeyCode::from_char('d'), vec![]));
    let result = resolver.add_key(Key::new(KeyCode::from_char('q'), vec![]));

    println!("{:?}", result);

    assert_eq!(KeySequence::Completed("dq".to_string()), result.1);
    assert!(result.0.is_empty());
}

#[test]
fn add_and_resolve_key_normal_dd() {
    let mut resolver = MessageResolver::default();
    resolver.mode = Mode::Normal;

    let _ = resolver.add_key(Key::new(KeyCode::from_char('d'), vec![]));
    let result = resolver.add_key(Key::new(KeyCode::from_char('d'), vec![]));

    println!("{:?}", result);

    assert_eq!(
        Some(&KeymapMessage::Buffer(BufferMessage::Modification(
            1,
            TextModification::DeleteLine
        ))),
        result.0.first()
    );
    assert_eq!(KeySequence::Completed("dd".to_string()), result.1);
    assert_eq!(1, result.0.len());
}

#[test]
fn add_and_resolve_key_normal_fq() {
    let mut resolver = MessageResolver::default();
    resolver.mode = Mode::Normal;

    let _ = resolver.add_key(Key::new(KeyCode::from_char('f'), vec![]));
    let result = resolver.add_key(Key::new(KeyCode::from_char('q'), vec![]));

    println!("{:?}", result);

    assert_eq!(
        Some(&KeymapMessage::Buffer(BufferMessage::MoveCursor(
            1,
            CursorDirection::FindForward('q')
        ))),
        result.0.first()
    );
    assert_eq!(KeySequence::Completed("fq".to_string()), result.1);
    assert_eq!(1, result.0.len());
}

#[test]
fn add_and_resolve_key_normal_dfq() {
    let mut resolver = MessageResolver::default();
    resolver.mode = Mode::Normal;

    let _ = resolver.add_key(Key::new(KeyCode::from_char('d'), vec![]));
    let _ = resolver.add_key(Key::new(KeyCode::from_char('f'), vec![]));
    let result = resolver.add_key(Key::new(KeyCode::from_char('q'), vec![]));

    println!("{:?}", result);

    assert_eq!(
        Some(&KeymapMessage::Buffer(BufferMessage::Modification(
            1,
            TextModification::DeleteMotion(1, CursorDirection::FindForward('q'))
        ))),
        result.0.first()
    );
    assert_eq!(KeySequence::Completed("dfq".to_string()), result.1);
    assert_eq!(1, result.0.len());
}

#[test]
fn add_and_resolve_key_normal_10fq() {
    let mut resolver = MessageResolver::default();
    resolver.mode = Mode::Normal;

    let _ = resolver.add_key(Key::new(KeyCode::from_char('1'), vec![]));
    let _ = resolver.add_key(Key::new(KeyCode::from_char('0'), vec![]));
    let _ = resolver.add_key(Key::new(KeyCode::from_char('f'), vec![]));
    let result = resolver.add_key(Key::new(KeyCode::from_char('q'), vec![]));

    println!("{:?}", result);

    assert_eq!(
        Some(&KeymapMessage::Buffer(BufferMessage::MoveCursor(
            10,
            CursorDirection::FindForward('q')
        ))),
        result.0.first()
    );
    assert_eq!(KeySequence::Completed("10fq".to_string()), result.1);
    assert_eq!(1, result.0.len());
}

#[test]
fn add_and_resolve_key_normal_d0() {
    let mut resolver = MessageResolver::default();
    resolver.mode = Mode::Normal;

    let _ = resolver.add_key(Key::new(KeyCode::from_char('d'), vec![]));
    let result = resolver.add_key(Key::new(KeyCode::from_char('0'), vec![]));

    println!("{:?}", result);

    assert_eq!(
        Some(&KeymapMessage::Buffer(BufferMessage::Modification(
            1,
            TextModification::DeleteMotion(1, CursorDirection::LineStart)
        ))),
        result.0.first()
    );
    assert_eq!(KeySequence::Completed("d0".to_string()), result.1);
    assert_eq!(1, result.0.len());
}

#[test]
fn add_and_resolve_key_command_q() {
    let mut resolver = MessageResolver::default();
    resolver.mode = Mode::Command(CommandMode::Command);

    let result = resolver.add_key(Key::new(KeyCode::from_char('q'), vec![]));

    println!("{:?}", result);

    assert_eq!(
        Some(&KeymapMessage::Buffer(BufferMessage::Modification(
            1,
            TextModification::Insert("q".to_string())
        ))),
        result.0.first()
    );
    assert_eq!(KeySequence::Completed("q".to_string()), result.1);
    assert_eq!(1, result.0.len());
}

#[test]
fn add_and_resolve_key_navigation_q() {
    let mut resolver = MessageResolver::default();
    resolver.mode = Mode::Navigation;

    let result = resolver.add_key(Key::new(
        KeyCode::from_char('q'),
        vec![KeyModifier::Ctrl, KeyModifier::Shift],
    ));

    println!("{:?}", result);

    assert_eq!(KeySequence::Completed("<C-Q>".to_string()), result.1);
    assert!(result.0.is_empty());
}

#[test]
fn add_and_resolve_key_navigation_10h() {
    let mut resolver = MessageResolver::default();

    let _ = resolver.add_key(Key::new(KeyCode::from_char('1'), vec![]));
    let _ = resolver.add_key(Key::new(KeyCode::from_char('0'), vec![]));
    let result = resolver.add_key(Key::new(KeyCode::from_char('h'), vec![]));

    println!("{:?}", result);

    assert_eq!(Some(&KeymapMessage::NavigateToParent), result.0.first());
    assert_eq!(Some(&KeymapMessage::NavigateToParent), result.0.get(1));
    assert_eq!(Some(&KeymapMessage::NavigateToParent), result.0.get(2));
    assert_eq!(Some(&KeymapMessage::NavigateToParent), result.0.get(3));
    assert_eq!(Some(&KeymapMessage::NavigateToParent), result.0.get(4));
    assert_eq!(Some(&KeymapMessage::NavigateToParent), result.0.get(5));
    assert_eq!(Some(&KeymapMessage::NavigateToParent), result.0.get(6));
    assert_eq!(Some(&KeymapMessage::NavigateToParent), result.0.get(7));
    assert_eq!(Some(&KeymapMessage::NavigateToParent), result.0.get(8));
    assert_eq!(Some(&KeymapMessage::NavigateToParent), result.0.get(9));
    assert_eq!(KeySequence::Completed("10h".to_string()), result.1);
    assert_eq!(10, result.0.len());
}

#[test]
fn add_and_resolve_key_navigation_yy() {
    let mut resolver = MessageResolver::default();

    let _ = resolver.add_key(Key::new(KeyCode::from_char('y'), vec![]));
    let result = resolver.add_key(Key::new(KeyCode::from_char('y'), vec![]));

    println!("{:?}", result);

    assert_eq!(Some(&KeymapMessage::YankToJunkYard(1)), result.0.first());
    assert_eq!(KeySequence::Completed("yy".to_string()), result.1);
    assert_eq!(1, result.0.len());
}

#[test]
fn add_and_resolve_key_navigation_10yy() {
    let mut resolver = MessageResolver::default();

    let _ = resolver.add_key(Key::new(KeyCode::from_char('1'), vec![]));
    let _ = resolver.add_key(Key::new(KeyCode::from_char('0'), vec![]));
    let _ = resolver.add_key(Key::new(KeyCode::from_char('y'), vec![]));
    let result = resolver.add_key(Key::new(KeyCode::from_char('y'), vec![]));

    println!("{:?}", result);

    assert_eq!(Some(&KeymapMessage::YankToJunkYard(10)), result.0.first());
    assert_eq!(KeySequence::Completed("10yy".to_string()), result.1);
    assert_eq!(1, result.0.len());
}

#[test]
fn add_and_resolve_key_normal_0() {
    let mut resolver = MessageResolver::default();
    resolver.mode = Mode::Normal;

    let result = resolver.add_key(Key::new(KeyCode::from_char('0'), vec![]));

    println!("{:?}", result);

    assert_eq!(
        Some(&KeymapMessage::Buffer(BufferMessage::MoveCursor(
            1,
            CursorDirection::LineStart
        ))),
        result.0.first()
    );
    assert_eq!(KeySequence::Completed("0".to_string()), result.1);
    assert_eq!(1, result.0.len());
}

#[test]
fn add_and_resolve_key_normal_d10fq() {
    let mut resolver = MessageResolver::default();
    resolver.mode = Mode::Normal;

    let _ = resolver.add_key(Key::new(KeyCode::from_char('d'), vec![]));
    let _ = resolver.add_key(Key::new(KeyCode::from_char('1'), vec![]));
    let _ = resolver.add_key(Key::new(KeyCode::from_char('0'), vec![]));
    let _ = resolver.add_key(Key::new(KeyCode::from_char('f'), vec![]));
    let result = resolver.add_key(Key::new(KeyCode::from_char('q'), vec![]));

    println!("{:?}", result);

    assert_eq!(
        Some(&KeymapMessage::Buffer(BufferMessage::Modification(
            1,
            TextModification::DeleteMotion(10, CursorDirection::FindForward('q'))
        ))),
        result.0.first()
    );
    assert_eq!(KeySequence::Completed("d10fq".to_string()), result.1);
    assert_eq!(1, result.0.len());
}

#[test]
fn add_and_resolve_key_normal_10d10fq() {
    let mut resolver = MessageResolver::default();
    resolver.mode = Mode::Normal;

    let _ = resolver.add_key(Key::new(KeyCode::from_char('1'), vec![]));
    let _ = resolver.add_key(Key::new(KeyCode::from_char('0'), vec![]));
    let _ = resolver.add_key(Key::new(KeyCode::from_char('d'), vec![]));
    let _ = resolver.add_key(Key::new(KeyCode::from_char('1'), vec![]));
    let _ = resolver.add_key(Key::new(KeyCode::from_char('0'), vec![]));
    let _ = resolver.add_key(Key::new(KeyCode::from_char('f'), vec![]));
    let result = resolver.add_key(Key::new(KeyCode::from_char('q'), vec![]));

    println!("{:?}", result);

    assert_eq!(
        Some(&KeymapMessage::Buffer(BufferMessage::Modification(
            10,
            TextModification::DeleteMotion(10, CursorDirection::FindForward('q'))
        ))),
        result.0.first()
    );
    assert_eq!(KeySequence::Completed("10d10fq".to_string()), result.1);
    assert_eq!(1, result.0.len());
}

#[test]
fn add_and_resolve_key_normal_10colon() {
    let mut resolver = MessageResolver::default();

    let _ = resolver.add_key(Key::new(KeyCode::from_char('1'), vec![]));
    let _ = resolver.add_key(Key::new(KeyCode::from_char('0'), vec![]));
    let result = resolver.add_key(Key::new(KeyCode::from_char(':'), vec![]));

    println!("{:?}", result);

    assert_eq!(
        Some(&KeymapMessage::Buffer(BufferMessage::ChangeMode(
            Mode::Navigation,
            Mode::Command(CommandMode::Command)
        ))),
        result.0.first()
    );
    assert_eq!(KeySequence::Completed("10:".to_string()), result.1);
    assert_eq!(1, result.0.len());
}

#[test]
fn add_and_resolve_key_normal_qa_and_q() {
    let mut resolver = MessageResolver::default();
    resolver.mode = Mode::Normal;

    let _ = resolver.add_key(Key::new(KeyCode::from_char('q'), vec![]));
    let result = resolver.add_key(Key::new(KeyCode::from_char('a'), vec![]));

    println!("{:?}", result);

    assert_eq!(Some(&KeymapMessage::StartMacro('a')), result.0.first());
    assert_eq!(KeySequence::Completed("qa".to_string()), result.1);
    assert_eq!(1, result.0.len());

    let result = resolver.add_key(Key::new(KeyCode::from_char('q'), vec![]));

    println!("{:?}", result);

    assert_eq!(Some(&KeymapMessage::StopMacro), result.0.first());
    assert_eq!(KeySequence::Completed("q".to_string()), result.1);
    assert_eq!(1, result.0.len());
}
