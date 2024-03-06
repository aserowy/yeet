use yeet_keymap::{
    key::{Key, KeyCode},
    message::{Buffer, CommandMode, CursorDirection, Message, Mode, TextModification},
    MessageResolver,
};

#[test]
fn add_and_resolve_key_navigation_colon() {
    let mut resolver = MessageResolver::default();
    let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char(':'), vec![]));

    println!("{:?}", messages);

    assert_eq!(
        Some(&Message::Buffer(Buffer::ChangeMode(
            Mode::Navigation,
            Mode::Command(CommandMode::Command)
        ))),
        messages.first()
    );
    assert_eq!(
        Some(&Message::KeySequenceChanged("".to_string())),
        messages.last()
    );
    assert_eq!(2, messages.len());
}

#[test]
fn add_and_resolve_key_navigation_d() {
    let mut resolver = MessageResolver::default();
    let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('d'), vec![]));

    println!("{:?}", messages);

    assert_eq!(
        Some(&Message::KeySequenceChanged("d".to_string())),
        messages.first()
    );
    assert_eq!(1, messages.len());
}

#[test]
fn add_and_resolve_key_navigation_dq() {
    let mut resolver = MessageResolver::default();
    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('d'), vec![]));
    let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('q'), vec![]));

    println!("{:?}", messages);

    assert_eq!(
        Some(&Message::KeySequenceChanged("".to_string())),
        messages.first()
    );
    assert_eq!(1, messages.len());
}

#[test]
fn add_and_resolve_key_normal_dd() {
    let mut resolver = MessageResolver::default();
    resolver.mode = Mode::Normal;

    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('d'), vec![]));
    let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('d'), vec![]));

    println!("{:?}", messages);

    assert_eq!(
        Some(&Message::Buffer(Buffer::Modification(
            1,
            TextModification::DeleteLine
        ))),
        messages.first()
    );
    assert_eq!(
        Some(&Message::KeySequenceChanged("".to_string())),
        messages.last()
    );
    assert_eq!(2, messages.len());
}

#[test]
fn add_and_resolve_key_normal_fq() {
    let mut resolver = MessageResolver::default();
    resolver.mode = Mode::Normal;

    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('f'), vec![]));
    let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('q'), vec![]));

    println!("{:?}", messages);

    assert_eq!(
        Some(&Message::Buffer(Buffer::MoveCursor(
            1,
            CursorDirection::FindForward('q')
        ))),
        messages.first()
    );
    assert_eq!(
        Some(&Message::KeySequenceChanged("".to_string())),
        messages.last()
    );
    assert_eq!(2, messages.len());
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
        Some(&Message::Buffer(Buffer::Modification(
            1,
            TextModification::DeleteMotion(1, CursorDirection::FindForward('q'))
        ))),
        messages.first()
    );
    assert_eq!(
        Some(&Message::KeySequenceChanged("".to_string())),
        messages.last()
    );
    assert_eq!(2, messages.len());
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
        Some(&Message::Buffer(Buffer::MoveCursor(
            10,
            CursorDirection::FindForward('q')
        ))),
        messages.first()
    );
    assert_eq!(
        Some(&Message::KeySequenceChanged("".to_string())),
        messages.last()
    );
    assert_eq!(2, messages.len());
}

#[test]
fn add_and_resolve_key_normal_d0() {
    let mut resolver = MessageResolver::default();
    resolver.mode = Mode::Normal;

    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('d'), vec![]));
    let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('0'), vec![]));

    println!("{:?}", messages);

    assert_eq!(
        Some(&Message::Buffer(Buffer::Modification(
            1,
            TextModification::DeleteMotion(1, CursorDirection::LineStart)
        ))),
        messages.first()
    );
    assert_eq!(
        Some(&Message::KeySequenceChanged("".to_string())),
        messages.last()
    );
    assert_eq!(2, messages.len());
}

#[test]
fn add_and_resolve_key_command_q() {
    let mut resolver = MessageResolver::default();
    resolver.mode = Mode::Command(CommandMode::Command);

    let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('q'), vec![]));

    println!("{:?}", messages);

    assert_eq!(
        Some(&Message::Buffer(Buffer::Modification(
            1,
            TextModification::Insert("q".to_string())
        ))),
        messages.first()
    );
    assert_eq!(
        Some(&Message::KeySequenceChanged("".to_string())),
        messages.last()
    );
    assert_eq!(2, messages.len());
}

#[test]
fn add_and_resolve_key_navigation_q() {
    let mut resolver = MessageResolver::default();
    resolver.mode = Mode::Navigation;

    let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('q'), vec![]));

    println!("{:?}", messages);

    assert_eq!(
        Some(&Message::KeySequenceChanged("".to_string())),
        messages.first()
    );
    assert_eq!(1, messages.len());
}

#[test]
fn add_and_resolve_key_navigation_10h() {
    let mut resolver = MessageResolver::default();

    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('1'), vec![]));
    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('0'), vec![]));
    let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('h'), vec![]));

    println!("{:?}", messages);

    assert_eq!(Some(&Message::NavigateToParent), messages.first());
    assert_eq!(Some(&Message::NavigateToParent), messages.get(1));
    assert_eq!(Some(&Message::NavigateToParent), messages.get(2));
    assert_eq!(Some(&Message::NavigateToParent), messages.get(3));
    assert_eq!(Some(&Message::NavigateToParent), messages.get(4));
    assert_eq!(Some(&Message::NavigateToParent), messages.get(5));
    assert_eq!(Some(&Message::NavigateToParent), messages.get(6));
    assert_eq!(Some(&Message::NavigateToParent), messages.get(7));
    assert_eq!(Some(&Message::NavigateToParent), messages.get(8));
    assert_eq!(Some(&Message::NavigateToParent), messages.get(9));
    assert_eq!(
        Some(&Message::KeySequenceChanged("".to_string())),
        messages.last()
    );
    assert_eq!(11, messages.len());
}

#[test]
fn add_and_resolve_key_navigation_yy() {
    let mut resolver = MessageResolver::default();

    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('y'), vec![]));
    let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('y'), vec![]));

    println!("{:?}", messages);

    assert_eq!(Some(&Message::YankToJunkYard(1)), messages.first());
    assert_eq!(
        Some(&Message::KeySequenceChanged("".to_string())),
        messages.last()
    );
    assert_eq!(2, messages.len());
}

#[test]
fn add_and_resolve_key_navigation_10yy() {
    let mut resolver = MessageResolver::default();

    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('1'), vec![]));
    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('0'), vec![]));
    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('y'), vec![]));
    let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('y'), vec![]));

    println!("{:?}", messages);

    assert_eq!(Some(&Message::YankToJunkYard(10)), messages.first());
    assert_eq!(
        Some(&Message::KeySequenceChanged("".to_string())),
        messages.last()
    );
    assert_eq!(2, messages.len());
}

#[test]
fn add_and_resolve_key_normal_0() {
    let mut resolver = MessageResolver::default();
    resolver.mode = Mode::Normal;

    let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('0'), vec![]));

    println!("{:?}", messages);

    assert_eq!(
        Some(&Message::Buffer(Buffer::MoveCursor(
            1,
            CursorDirection::LineStart
        ))),
        messages.first()
    );
    assert_eq!(
        Some(&Message::KeySequenceChanged("".to_string())),
        messages.last()
    );
    assert_eq!(2, messages.len());
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
        Some(&Message::Buffer(Buffer::Modification(
            1,
            TextModification::DeleteMotion(10, CursorDirection::FindForward('q'))
        ))),
        messages.first()
    );
    assert_eq!(
        Some(&Message::KeySequenceChanged("".to_string())),
        messages.last()
    );
    assert_eq!(2, messages.len());
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
        Some(&Message::Buffer(Buffer::Modification(
            10,
            TextModification::DeleteMotion(10, CursorDirection::FindForward('q'))
        ))),
        messages.first()
    );
    assert_eq!(
        Some(&Message::KeySequenceChanged("".to_string())),
        messages.last()
    );
    assert_eq!(2, messages.len());
}

#[test]
fn add_and_resolve_key_normal_10colon() {
    let mut resolver = MessageResolver::default();

    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('1'), vec![]));
    let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('0'), vec![]));
    let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char(':'), vec![]));

    println!("{:?}", messages);

    assert_eq!(
        Some(&Message::Buffer(Buffer::ChangeMode(
            Mode::Navigation,
            Mode::Command(CommandMode::Command)
        ))),
        messages.first()
    );
    assert_eq!(
        Some(&Message::KeySequenceChanged("".to_string())),
        messages.last()
    );
    assert_eq!(2, messages.len());
}
