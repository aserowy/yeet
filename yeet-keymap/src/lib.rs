use buffer::KeyBuffer;
use key::{Key, KeyCode};
use map::KeyMap;
use message::{Binding, BindingKind, CursorDirection, Message, Mode, TextModification};
use tree::KeyTree;

use crate::message::Buffer;

mod buffer;
pub mod conversion;
pub mod key;
mod map;
pub mod message;
mod tree;

#[derive(Debug, thiserror::Error, PartialEq)]
enum KeyMapError {
    #[error("Key sequence is incomplete.")]
    KeySequenceIncomplete,
    #[error("Failed to add mapping for mode {0}")]
    ModeUnresolvable(String),
    #[error("Failed to resolve valid binding.")]
    NoValidBindingFound,
}

#[derive(Debug)]
pub struct MessageResolver {
    buffer: KeyBuffer,
    tree: KeyTree,
    pub mode: Mode,
}

impl Default for MessageResolver {
    fn default() -> Self {
        Self {
            buffer: KeyBuffer::default(),
            tree: KeyMap::default().into_tree(),
            mode: Mode::default(),
        }
    }
}

impl MessageResolver {
    pub fn add_and_resolve(&mut self, key: Key) -> Vec<Message> {
        let keys = self.buffer.get_keys();
        if key.code == KeyCode::Esc && !keys.is_empty() {
            self.buffer.clear();
            return vec![Message::KeySequenceChanged(self.buffer.to_string())];
        }

        self.buffer.add_key(key);
        println!("buffer: {:?}", self.buffer);

        let keys = self.buffer.get_keys();
        let binding = resolve_binding(&self.tree, &self.mode, &keys, None);

        let mut messages = match binding {
            Ok(Some(binding)) => {
                self.buffer.clear();
                get_messages_from_binding(&self.mode, binding)
            }
            Ok(None) => Vec::new(),
            Err(KeyMapError::KeySequenceIncomplete) => Vec::new(),
            Err(_) => {
                let messages = if get_passthrough_by_mode(&self.mode) {
                    let message = TextModification::Insert(self.buffer.to_string());
                    vec![Message::Buffer(Buffer::Modification(1, message))]
                } else {
                    Vec::new()
                };

                self.buffer.clear();
                messages
            }
        };

        messages.push(Message::KeySequenceChanged(self.buffer.to_string()));
        messages
    }
}

fn get_messages_from_binding(mode: &Mode, binding: Binding) -> Vec<Message> {
    let mut messages = Vec::new();
    if let Some(md) = &binding.force {
        messages.push(Message::Buffer(Buffer::ChangeMode(
            mode.clone(),
            md.clone(),
        )));
    };

    let repeat = match binding.repeat {
        Some(it) => it,
        None => 1,
    };

    match &binding.kind {
        BindingKind::Message(msg) => messages.extend(get_repeated_message(repeat, msg)),
        BindingKind::Modification(mdf) => {
            messages.push(Message::Buffer(Buffer::Modification(1, mdf.clone())))
        }
        BindingKind::Motion(mtn) => {
            messages.push(Message::Buffer(Buffer::MoveCursor(repeat, mtn.clone())))
        }
        BindingKind::None => {}
        BindingKind::Raw(_) | BindingKind::Repeat | BindingKind::RepeatOrMotion(_) => {
            unreachable!()
        }
    }

    messages
}

fn get_repeated_message(repeat: usize, msg: &Message) -> Vec<Message> {
    let mut messages = Vec::new();
    match msg {
        Message::YankSelected(_) => messages.push(Message::YankSelected(repeat)),
        _ => {
            for _ in 0..repeat {
                messages.push(msg.clone());
            }
        }
    }

    messages
}

fn get_passthrough_by_mode(mode: &Mode) -> bool {
    match mode {
        Mode::Command | Mode::Insert => true,
        Mode::Navigation | Mode::Normal => false,
    }
}

fn resolve_binding(
    tree: &KeyTree,
    mode: &Mode,
    keys: &Vec<Key>,
    before: Option<&Binding>,
) -> Result<Option<Binding>, KeyMapError> {
    if keys.is_empty() {
        return Ok(None);
    }

    if let Some(before) = before {
        if let Some(message::NextBindingKind::Raw) = before.expects {
            let key = match keys.first() {
                Some(it) => it,
                None => {
                    return Err(KeyMapError::KeySequenceIncomplete);
                }
            };

            let string = key.to_string();
            let chars: Vec<_> = string.chars().collect();
            if chars.len() != 1 {
                return Err(KeyMapError::NoValidBindingFound);
            }

            return Ok(Some(Binding {
                kind: BindingKind::Raw(chars[0]),
                ..Default::default()
            }));
        }
    }

    let (mut binding, unused_keys) = {
        let (mut binding, unused_keys) = tree.get_binding(mode, &keys)?;

        let binding = if let BindingKind::RepeatOrMotion(motion) = binding.kind {
            if let Some(before) = before {
                if let BindingKind::Repeat = before.kind {
                    binding.kind = BindingKind::Repeat;
                    binding
                } else {
                    Binding::from_motion(motion)
                }
            } else {
                Binding::from_motion(motion)
            }
        } else {
            binding
        };

        (binding, unused_keys)
    };

    let mut next = match resolve_binding(tree, mode, &unused_keys, Some(&binding))? {
        Some(it) => it,
        None => {
            if binding.expects.is_some() {
                return Err(KeyMapError::KeySequenceIncomplete);
            } else if let BindingKind::Repeat = binding.kind {
                return Err(KeyMapError::KeySequenceIncomplete);
            } else {
                return Ok(Some(binding));
            }
        }
    };

    let result = if binding.expects.is_some() {
        binding.kind = combine(&binding, &next)?;
        binding
    } else if let BindingKind::Repeat = binding.kind {
        next.repeat = get_repeat(&binding, &next);
        next
    } else {
        binding
    };

    Ok(Some(result))
}

fn combine(current: &Binding, next: &Binding) -> Result<BindingKind, KeyMapError> {
    match (&current.kind, &next.kind) {
        (BindingKind::Motion(direction), BindingKind::Raw(raw)) => {
            let direction = match direction {
                CursorDirection::FindBackward(_) => CursorDirection::FindBackward(*raw),
                CursorDirection::FindForward(_) => CursorDirection::FindForward(*raw),
                CursorDirection::TillBackward(_) => CursorDirection::TillBackward(*raw),
                CursorDirection::TillForward(_) => CursorDirection::TillForward(*raw),
                _ => return Err(KeyMapError::NoValidBindingFound),
            };

            Ok(BindingKind::Motion(direction))
        }
        (BindingKind::Modification(mdf), BindingKind::Motion(mtn)) => {
            let repeat = match next.repeat {
                Some(it) => it,
                None => 1,
            };

            let modification = match mdf {
                TextModification::DeleteMotion(_, _) => {
                    TextModification::DeleteMotion(repeat, mtn.clone())
                }
                _ => return Err(KeyMapError::NoValidBindingFound),
            };

            Ok(BindingKind::Modification(modification))
        }
        (_, _) => Err(KeyMapError::NoValidBindingFound),
    }
}

fn get_repeat(current: &Binding, next: &Binding) -> Option<usize> {
    if !next.repeatable {
        return next.repeat;
    }

    let current_repeat = match current.repeat {
        Some(it) => it,
        None => return next.repeat,
    };

    let repeat = match next.repeat {
        Some(it) => {
            let repeat_len = it.to_string().len();
            let pow = match 10_usize.checked_pow(repeat_len as u32) {
                Some(it) => it,
                None => {
                    return next.repeat;
                }
            };

            current_repeat * pow + it
        }
        None => current_repeat,
    };

    Some(repeat)
}

mod test {
    #[test]
    fn add_and_resolve_key_navigation_colon() {
        use crate::key::{Key, KeyCode};
        use crate::message::{Buffer, Message, Mode};

        let mut resolver = super::MessageResolver::default();
        let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char(':'), vec![]));

        println!("{:?}", messages);

        assert_eq!(
            Some(&Message::Buffer(Buffer::ChangeMode(
                Mode::Navigation,
                Mode::Command
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
        use crate::key::{Key, KeyCode};
        use crate::message::Message;

        let mut resolver = super::MessageResolver::default();
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
        use crate::key::{Key, KeyCode};
        use crate::message::Message;

        let mut resolver = super::MessageResolver::default();
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
        use crate::key::{Key, KeyCode};
        use crate::message::{Buffer, Message, Mode, TextModification};

        let mut resolver = super::MessageResolver::default();
        resolver.mode = Mode::Normal;

        let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('d'), vec![]));
        let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('d'), vec![]));

        println!("{:?}", messages);

        assert_eq!(
            Some(&Message::Buffer(Buffer::Modification(
                1,
                TextModification::DeleteLineOnCursor
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
        use crate::key::{Key, KeyCode};
        use crate::message::{Buffer, CursorDirection, Message, Mode};

        let mut resolver = super::MessageResolver::default();
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
        use crate::key::{Key, KeyCode};
        use crate::message::{Buffer, CursorDirection, Message, Mode, TextModification};

        let mut resolver = super::MessageResolver::default();
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
        use crate::key::{Key, KeyCode};
        use crate::message::{Buffer, CursorDirection, Message, Mode};

        let mut resolver = super::MessageResolver::default();
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
        use crate::key::{Key, KeyCode};
        use crate::message::{Buffer, CursorDirection, Message, Mode, TextModification};

        let mut resolver = super::MessageResolver::default();
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
        use crate::key::{Key, KeyCode};
        use crate::message::{Buffer, Message, Mode, TextModification};

        let mut resolver = super::MessageResolver::default();
        resolver.mode = Mode::Command;

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
        use crate::key::{Key, KeyCode};
        use crate::message::{Message, Mode};

        let mut resolver = super::MessageResolver::default();
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
        use crate::key::{Key, KeyCode};
        use crate::message::Message;

        let mut resolver = super::MessageResolver::default();

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
        use crate::key::{Key, KeyCode};
        use crate::message::Message;

        let mut resolver = super::MessageResolver::default();

        let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('y'), vec![]));
        let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('y'), vec![]));

        println!("{:?}", messages);

        assert_eq!(Some(&Message::YankSelected(1)), messages.first());
        assert_eq!(
            Some(&Message::KeySequenceChanged("".to_string())),
            messages.last()
        );
        assert_eq!(2, messages.len());
    }

    #[test]
    fn add_and_resolve_key_navigation_10yy() {
        use crate::key::{Key, KeyCode};
        use crate::message::Message;

        let mut resolver = super::MessageResolver::default();

        let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('1'), vec![]));
        let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('0'), vec![]));
        let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('y'), vec![]));
        let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('y'), vec![]));

        println!("{:?}", messages);

        assert_eq!(Some(&Message::YankSelected(10)), messages.first());
        assert_eq!(
            Some(&Message::KeySequenceChanged("".to_string())),
            messages.last()
        );
        assert_eq!(2, messages.len());
    }

    #[test]
    fn add_and_resolve_key_normal_0() {
        use crate::key::{Key, KeyCode};
        use crate::message::{Buffer, CursorDirection, Message, Mode};

        let mut resolver = super::MessageResolver::default();
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
}
