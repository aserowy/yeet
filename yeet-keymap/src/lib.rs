use buffer::KeyBuffer;
use key::{Key, KeyCode};
use map::KeyMap;
use message::{Binding, BindingKind, CursorDirection, Message, Mode};
use tree::KeyTree;

use crate::message::{Buffer, TextModification};

mod buffer;
pub mod conversion;
pub mod key;
mod map;
pub mod message;
mod tree;

#[derive(Debug, thiserror::Error)]
enum KeyMapError {
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

        let keys = self.buffer.get_keys();
        let bindings = match self.tree.get_bindings(&self.mode, &keys) {
            Ok(Some(bindings)) => bindings,
            Ok(None) => return vec![Message::KeySequenceChanged(self.buffer.to_string())],
            Err(err) => {
                self.buffer.clear();
                println!("{:?}", err);
                return vec![Message::KeySequenceChanged("".to_string())];
            }
        };

        println!("{:?}", bindings);
        let mut messages = if bindings.is_empty() {
            let messages = if get_passthrough_by_mode(&self.mode) {
                let message = TextModification::Insert(self.buffer.to_string());
                vec![Message::Buffer(Buffer::Modification(1, message))]
            } else {
                Vec::new()
            };

            self.buffer.clear();
            messages
        } else {
            let messages = match get_messages_from_bindings(bindings, &mut self.mode) {
                Some(msgs) => msgs,
                None => {
                    self.buffer.clear();
                    Vec::new()
                }
            };

            if !messages.is_empty() {
                self.buffer.clear();
            }

            messages
        };

        println!("{:?}", messages);
        messages.push(Message::KeySequenceChanged(self.buffer.to_string()));
        messages
    }
}

fn get_passthrough_by_mode(mode: &Mode) -> bool {
    match mode {
        Mode::Command => true,
        Mode::Insert => true,
        Mode::Navigation => false,
        Mode::Normal => false,
    }
}

fn get_messages_from_bindings(bindings: Vec<Binding>, mode: &mut Mode) -> Option<Vec<Message>> {
    let mut consolidated: Vec<Binding> = Vec::new();
    println!("{:?}", bindings);
    for binding in bindings.iter().rev() {
        if let Some(expects) = &binding.expects {
            let last = match consolidated.last_mut() {
                Some(it) => it,
                None => return Some(Vec::new()),
            };

            if !last.equals(expects) {
                println!("Expects not equal");
                return None;
            }

            last.kind = combine(&binding, last)?;
        } else {
            handle_binding(&mut consolidated, binding);
        }
    }

    Some(map_bindings(consolidated.iter().rev().collect(), mode))
}

fn handle_binding(consolidated: &mut Vec<Binding>, binding: &Binding) {
    match &binding.kind {
        // TODO: test repeat or motion on input start to convert to motion
        BindingKind::Repeat | BindingKind::RepeatOrMotion(_) => {
            let last = match consolidated.last_mut() {
                Some(it) => it,
                None => {
                    consolidated.push(binding.clone());
                    return;
                }
            };

            if !last.repeatable {
                return;
            }

            let front = match binding.repeat {
                Some(it) => it,
                None => return,
            };

            let repeat = match last.repeat {
                Some(it) => {
                    let repeat_len = it.to_string().len();
                    front * 10 ^ repeat_len + it
                }
                None => front,
            };

            last.repeat = Some(repeat);
        }
        _ => consolidated.push(binding.clone()),
    }
}

fn combine(current: &Binding, last: &Binding) -> Option<BindingKind> {
    match (&current.kind, &last.kind) {
        (BindingKind::Motion(direction), BindingKind::Raw(raw)) => {
            let direction = match direction {
                CursorDirection::FindBackward(_) => CursorDirection::FindBackward(*raw),
                CursorDirection::FindForward(_) => CursorDirection::FindForward(*raw),
                CursorDirection::TillBackward(_) => CursorDirection::TillBackward(*raw),
                CursorDirection::TillForward(_) => CursorDirection::TillForward(*raw),
                _ => return None,
            };

            Some(BindingKind::Motion(direction))
        }
        (BindingKind::Modification(mdf), BindingKind::Motion(mtn))
        | (BindingKind::Modification(mdf), BindingKind::RepeatOrMotion(mtn)) => {
            let repeat = match last.repeat {
                Some(it) => it,
                None => 1,
            };

            let modification = match mdf {
                TextModification::DeleteMotion(_, _) => {
                    TextModification::DeleteMotion(repeat, mtn.clone())
                }
                _ => return None,
            };

            Some(BindingKind::Modification(modification))
        }
        (_, _) => None,
    }
}

fn map_bindings(bindings: Vec<&Binding>, mode: &Mode) -> Vec<Message> {
    let mut messages = Vec::new();
    for binding in bindings {
        if let Some(md) = &binding.force {
            messages.push(Message::Buffer(Buffer::ChangeMode(
                mode.clone(),
                md.clone(),
            )));
        }

        // TODO: handle repeat value
        match &binding.kind {
            BindingKind::Message(msg) => messages.push(msg.clone()),
            BindingKind::Modification(mdf) => {
                messages.push(Message::Buffer(Buffer::Modification(1, mdf.clone())))
            }
            BindingKind::Motion(mtn) => {
                messages.push(Message::Buffer(Buffer::MoveCursor(1, mtn.clone())))
            }
            BindingKind::None => {}
            BindingKind::Raw(_) | BindingKind::Repeat | BindingKind::RepeatOrMotion(_) => {
                unreachable!()
            }
        }
    }

    messages
}

mod test {
    #[test]
    fn add_and_resolve_key_with_binding_forcemode_only() {
        use crate::key::{Key, KeyCode};
        use crate::message::{Buffer, Message, Mode};

        let mut resolver = super::MessageResolver::default();
        let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char(':'), vec![]));

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
    fn add_and_resolve_key_with_expector() {
        use crate::key::{Key, KeyCode};
        use crate::message::Message;

        let mut resolver = super::MessageResolver::default();
        let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('d'), vec![]));

        assert_eq!(
            Some(&Message::KeySequenceChanged("d".to_string())),
            messages.first()
        );
        assert_eq!(1, messages.len());
    }

    #[test]
    fn add_and_resolve_key_with_expector_and_wrong_followup() {
        use crate::key::{Key, KeyCode};
        use crate::message::Message;

        let mut resolver = super::MessageResolver::default();
        let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('d'), vec![]));
        let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('q'), vec![]));

        assert_eq!(
            Some(&Message::KeySequenceChanged("".to_string())),
            messages.first()
        );
        assert_eq!(1, messages.len());
    }

    #[test]
    fn add_and_resolve_key_with_expector_and_hashmap_hit() {
        use crate::key::{Key, KeyCode};
        use crate::message::{Buffer, Message, Mode, TextModification};

        let mut resolver = super::MessageResolver::default();
        resolver.mode = Mode::Normal;
        let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('d'), vec![]));
        let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('d'), vec![]));

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
    fn add_and_resolve_key_with_expector_and_raw() {
        use crate::key::{Key, KeyCode};
        use crate::message::{Buffer, CursorDirection, Message, Mode};

        let mut resolver = super::MessageResolver::default();
        resolver.mode = Mode::Normal;

        let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('f'), vec![]));
        let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('q'), vec![]));

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
    fn add_and_resolve_key_with_expector_and_motion() {
        use crate::key::{Key, KeyCode};
        use crate::message::{Buffer, CursorDirection, Message, Mode, TextModification};

        let mut resolver = super::MessageResolver::default();
        resolver.mode = Mode::Normal;

        let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('d'), vec![]));
        let _ = resolver.add_and_resolve(Key::new(KeyCode::from_char('f'), vec![]));
        let messages = resolver.add_and_resolve(Key::new(KeyCode::from_char('q'), vec![]));

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
}
