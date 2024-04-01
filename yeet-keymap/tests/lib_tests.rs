use yeet_buffer::{
    message::{BufferMessage, CursorDirection, TextModification},
    model::{CommandMode, Mode},
};
use yeet_keymap::{
    key::{Key, KeyCode, KeyModifier},
    message::{KeySequence, Message},
    MessageResolver,
};

#[test]
fn add_and_resolve_key_navigation_colon() {
    let mut resolver = MessageResolver::default();
    let envelope = resolver.add_and_resolve(Key::new(KeyCode::from_char(':'), vec![]));

    println!("{:?}", envelope);

    assert_eq!(
        Some(&Message::Buffer(BufferMessage::ChangeMode(
            Mode::Navigation,
            Mode::Command(CommandMode::Command)
        ))),
        envelope.messages.first()
    );
    assert_eq!(KeySequence::Completed(":".to_string()), envelope.sequence);
    assert_eq!(1, envelope.messages.len());
}

#[test]
fn add_and_resolve_key_navigation_d() {
    let mut resolver = MessageResolver::default();
    let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('d'), vec![]));

    println!("{:?}", messages);

    assert_eq!(KeySequence::Changed("d".to_string()), messages.sequence);
    assert!(messages.messages.is_empty());
}

#[test]
fn add_and_resolve_key_navigation_dq() {
    let mut resolver = MessageResolver::default();
    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('d'), vec![]));
    let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('q'), vec![]));

    println!("{:?}", messages);

    assert_eq!(KeySequence::Completed("dq".to_string()), messages.sequence);
    assert!(messages.messages.is_empty());
}

#[test]
fn add_and_resolve_key_normal_dd() {
    let mut resolver = MessageResolver::default();
    resolver.mode = Mode::Normal;

    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('d'), vec![]));
    let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('d'), vec![]));

    println!("{:?}", messages);

    assert_eq!(
        Some(&Message::Buffer(BufferMessage::Modification(
            1,
            TextModification::DeleteLine
        ))),
        messages.messages.first()
    );
    assert_eq!(KeySequence::Completed("dd".to_string()), messages.sequence);
    assert_eq!(1, messages.messages.len());
}

#[test]
fn add_and_resolve_key_normal_fq() {
    let mut resolver = MessageResolver::default();
    resolver.mode = Mode::Normal;

    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('f'), vec![]));
    let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('q'), vec![]));

    println!("{:?}", messages);

    assert_eq!(
        Some(&Message::Buffer(BufferMessage::MoveCursor(
            1,
            CursorDirection::FindForward('q')
        ))),
        messages.messages.first()
    );
    assert_eq!(KeySequence::Completed("fq".to_string()), messages.sequence);
    assert_eq!(1, messages.messages.len());
}

#[test]
fn add_and_resolve_key_normal_dfq() {
    let mut resolver = MessageResolver::default();
    resolver.mode = Mode::Normal;

    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('d'), vec![]));
    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('f'), vec![]));
    let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('q'), vec![]));

    println!("{:?}", messages);

    assert_eq!(
        Some(&Message::Buffer(BufferMessage::Modification(
            1,
            TextModification::DeleteMotion(1, CursorDirection::FindForward('q'))
        ))),
        messages.messages.first()
    );
    assert_eq!(KeySequence::Completed("dfq".to_string()), messages.sequence);
    assert_eq!(1, messages.messages.len());
}

#[test]
fn add_and_resolve_key_normal_10fq() {
    let mut resolver = MessageResolver::default();
    resolver.mode = Mode::Normal;

    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('1'), vec![]));
    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('0'), vec![]));
    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('f'), vec![]));
    let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('q'), vec![]));

    println!("{:?}", messages);

    assert_eq!(
        Some(&Message::Buffer(BufferMessage::MoveCursor(
            10,
            CursorDirection::FindForward('q')
        ))),
        messages.messages.first()
    );
    assert_eq!(
        KeySequence::Completed("10fq".to_string()),
        messages.sequence
    );
    assert_eq!(1, messages.messages.len());
}

#[test]
fn add_and_resolve_key_normal_d0() {
    let mut resolver = MessageResolver::default();
    resolver.mode = Mode::Normal;

    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('d'), vec![]));
    let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('0'), vec![]));

    println!("{:?}", messages);

    assert_eq!(
        Some(&Message::Buffer(BufferMessage::Modification(
            1,
            TextModification::DeleteMotion(1, CursorDirection::LineStart)
        ))),
        messages.messages.first()
    );
    assert_eq!(KeySequence::Completed("d0".to_string()), messages.sequence);
    assert_eq!(1, messages.messages.len());
}

#[test]
fn add_and_resolve_key_command_q() {
    let mut resolver = MessageResolver::default();
    resolver.mode = Mode::Command(CommandMode::Command);

    let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('q'), vec![]));

    println!("{:?}", messages);

    assert_eq!(
        Some(&Message::Buffer(BufferMessage::Modification(
            1,
            TextModification::Insert("q".to_string())
        ))),
        messages.messages.first()
    );
    assert_eq!(KeySequence::Completed("q".to_string()), messages.sequence);
    assert_eq!(1, messages.messages.len());
}

#[test]
fn add_and_resolve_key_navigation_q() {
    let mut resolver = MessageResolver::default();
    resolver.mode = Mode::Navigation;

    let messages = resolver.add_and_resolve(Key::new(
        KeyCode::from_char('q'),
        vec![KeyModifier::Ctrl, KeyModifier::Shift],
    ));

    println!("{:?}", messages);

    assert_eq!(
        KeySequence::Completed("<C-Q>".to_string()),
        messages.sequence
    );
    assert!(messages.messages.is_empty());
}

#[test]
fn add_and_resolve_key_navigation_10h() {
    let mut resolver = MessageResolver::default();

    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('1'), vec![]));
    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('0'), vec![]));
    let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('h'), vec![]));

    println!("{:?}", messages);

    assert_eq!(Some(&Message::NavigateToParent), messages.messages.first());
    assert_eq!(Some(&Message::NavigateToParent), messages.messages.get(1));
    assert_eq!(Some(&Message::NavigateToParent), messages.messages.get(2));
    assert_eq!(Some(&Message::NavigateToParent), messages.messages.get(3));
    assert_eq!(Some(&Message::NavigateToParent), messages.messages.get(4));
    assert_eq!(Some(&Message::NavigateToParent), messages.messages.get(5));
    assert_eq!(Some(&Message::NavigateToParent), messages.messages.get(6));
    assert_eq!(Some(&Message::NavigateToParent), messages.messages.get(7));
    assert_eq!(Some(&Message::NavigateToParent), messages.messages.get(8));
    assert_eq!(Some(&Message::NavigateToParent), messages.messages.get(9));
    assert_eq!(KeySequence::Completed("10h".to_string()), messages.sequence);
    assert_eq!(10, messages.messages.len());
}

#[test]
fn add_and_resolve_key_navigation_yy() {
    let mut resolver = MessageResolver::default();

    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('y'), vec![]));
    let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('y'), vec![]));

    println!("{:?}", messages);

    assert_eq!(Some(&Message::YankToJunkYard(1)), messages.messages.first());
    assert_eq!(KeySequence::Completed("yy".to_string()), messages.sequence);
    assert_eq!(1, messages.messages.len());
}

#[test]
fn add_and_resolve_key_navigation_10yy() {
    let mut resolver = MessageResolver::default();

    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('1'), vec![]));
    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('0'), vec![]));
    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('y'), vec![]));
    let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('y'), vec![]));

    println!("{:?}", messages);

    assert_eq!(
        Some(&Message::YankToJunkYard(10)),
        messages.messages.first()
    );
    assert_eq!(
        KeySequence::Completed("10yy".to_string()),
        messages.sequence
    );
    assert_eq!(1, messages.messages.len());
}

#[test]
fn add_and_resolve_key_normal_0() {
    let mut resolver = MessageResolver::default();
    resolver.mode = Mode::Normal;

    let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('0'), vec![]));

    println!("{:?}", messages);

    assert_eq!(
        Some(&Message::Buffer(BufferMessage::MoveCursor(
            1,
            CursorDirection::LineStart
        ))),
        messages.messages.first()
    );
    assert_eq!(KeySequence::Completed("0".to_string()), messages.sequence);
    assert_eq!(1, messages.messages.len());
}

#[test]
fn add_and_resolve_key_normal_d10fq() {
    let mut resolver = MessageResolver::default();
    resolver.mode = Mode::Normal;

    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('d'), vec![]));
    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('1'), vec![]));
    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('0'), vec![]));
    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('f'), vec![]));
    let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('q'), vec![]));

    println!("{:?}", messages);

    assert_eq!(
        Some(&Message::Buffer(BufferMessage::Modification(
            1,
            TextModification::DeleteMotion(10, CursorDirection::FindForward('q'))
        ))),
        messages.messages.first()
    );
    assert_eq!(
        KeySequence::Completed("d10fq".to_string()),
        messages.sequence
    );
    assert_eq!(1, messages.messages.len());
}

#[test]
fn add_and_resolve_key_normal_10d10fq() {
    let mut resolver = MessageResolver::default();
    resolver.mode = Mode::Normal;

    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('1'), vec![]));
    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('0'), vec![]));
    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('d'), vec![]));
    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('1'), vec![]));
    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('0'), vec![]));
    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('f'), vec![]));
    let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('q'), vec![]));

    println!("{:?}", messages);

    assert_eq!(
        Some(&Message::Buffer(BufferMessage::Modification(
            10,
            TextModification::DeleteMotion(10, CursorDirection::FindForward('q'))
        ))),
        messages.messages.first()
    );
    assert_eq!(
        KeySequence::Completed("10d10fq".to_string()),
        messages.sequence
    );
    assert_eq!(1, messages.messages.len());
}

#[test]
fn add_and_resolve_key_normal_10colon() {
    let mut resolver = MessageResolver::default();

    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('1'), vec![]));
    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('0'), vec![]));
    let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char(':'), vec![]));

    println!("{:?}", messages);

    assert_eq!(
        Some(&Message::Buffer(BufferMessage::ChangeMode(
            Mode::Navigation,
            Mode::Command(CommandMode::Command)
        ))),
        messages.messages.first()
    );
    assert_eq!(KeySequence::Completed("10:".to_string()), messages.sequence);
    assert_eq!(1, messages.messages.len());
}
