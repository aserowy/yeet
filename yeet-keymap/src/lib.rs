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

    let repeat = binding.repeat.unwrap_or(1);
    match &binding.kind {
        BindingKind::Message(msg) => messages.extend(get_repeated_message(repeat, msg)),
        BindingKind::Modification(mdf) => {
            messages.push(Message::Buffer(Buffer::Modification(repeat, mdf.clone())))
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

// TODO: refactor!
fn resolve_binding(
    tree: &KeyTree,
    mode: &Mode,
    keys: &[Key],
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
        let (mut binding, unused_keys) = tree.get_binding(mode, keys)?;

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
            let repeat = next.repeat.unwrap_or(1);
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
