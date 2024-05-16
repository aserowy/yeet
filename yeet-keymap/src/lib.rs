use std::collections::{HashSet, VecDeque};

use buffer::KeyBuffer;
use key::{Key, KeyCode};
use map::KeyMap;
use message::{Binding, BindingKind, Envelope, KeySequence, Message, MessageSource};
use tree::KeyTree;
use yeet_buffer::{
    message::{BufferMessage, CursorDirection, TextModification},
    model::Mode,
};

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
    #[error("Failed to add mapping for mode {0}.")]
    ModeUnresolvable(String),
    #[error("Failed to resolve valid binding.")]
    NoValidBindingFound,
    #[error("No keys left.")]
    NoKeysRemaining,
}

#[derive()]
pub struct MessageResolver {
    buffer: KeyBuffer,
    pub mode: Mode,
    toggle: HashSet<String>,
    tree: KeyTree,
}

impl Default for MessageResolver {
    fn default() -> Self {
        Self {
            buffer: KeyBuffer::default(),
            mode: Mode::default(),
            toggle: HashSet::new(),
            tree: KeyMap::default().into_tree(),
        }
    }
}

impl MessageResolver {
    pub fn add_keys(&mut self, mut keys: VecDeque<Key>) -> Option<Envelope> {
        while let Some(key) = keys.pop_front() {
            let mut envelope = self.add_key(key);
            if matches!(envelope.sequence, KeySequence::Completed(_)) {
                let remaining_sequence = keys
                    .iter()
                    .map(|key| key.to_keycode_string())
                    .collect::<Vec<_>>()
                    .join("");

                if !remaining_sequence.is_empty() {
                    envelope
                        .messages
                        .insert(0, Message::ExecuteKeySequence(remaining_sequence));
                }

                return Some(envelope);
            }
        }
        None
    }

    pub fn add_key(&mut self, key: Key) -> Envelope {
        let keys = self.buffer.get_keys();
        if key.code == KeyCode::Esc && !keys.is_empty() {
            self.buffer.clear();
            return Envelope {
                messages: Vec::new(),
                sequence: KeySequence::Completed(format!(
                    "{}{}",
                    self.buffer.to_keycode_string(),
                    key.to_keycode_string()
                )),
                source: MessageSource::User,
            };
        }

        self.buffer.add_key(key);

        let keys = self.buffer.get_keys();
        let binding = resolve_binding(&self.tree, &self.mode, &mut self.toggle, &keys, None);
        let sequence = self.buffer.to_keycode_string();

        let (messages, sequence) = match binding {
            Ok(binding) => {
                if let Some((identifier, _)) = &binding.toggle {
                    self.toggle.insert(identifier.to_string());
                }

                self.buffer.clear();
                let messages = get_messages_from_binding(&self.mode, binding);
                (messages, KeySequence::Completed(sequence))
            }
            Err(KeyMapError::KeySequenceIncomplete) => (Vec::new(), KeySequence::Changed(sequence)),
            Err(_) => {
                let messages = if get_passthrough_by_mode(&self.mode) {
                    let message = TextModification::Insert(self.buffer.to_string());
                    vec![Message::Buffer(BufferMessage::Modification(1, message))]
                } else {
                    Vec::new()
                };

                self.buffer.clear();
                (messages, KeySequence::Completed(sequence))
            }
        };

        Envelope {
            messages,
            sequence,
            source: MessageSource::User,
        }
    }
}

fn resolve_binding(
    tree: &KeyTree,
    mode: &Mode,
    toggle: &mut HashSet<String>,
    keys: &[Key],
    before: Option<&Binding>,
) -> Result<Binding, KeyMapError> {
    if keys.is_empty() {
        return Err(KeyMapError::NoKeysRemaining);
    }

    if let Some(raw) = return_raw_if_expected(before, keys)? {
        return Ok(raw);
    }

    let (mut binding, unused_keys) = get_binding_by_keys(before, tree, mode, keys)?;
    if let Some((identifier, kind)) = &binding.toggle {
        if toggle.remove(identifier) {
            return Ok(Binding {
                kind: kind.clone(),
                ..Default::default()
            });
        }
    }

    let mut next = match resolve_binding(tree, mode, toggle, &unused_keys, Some(&binding)) {
        Ok(it) => it,
        Err(KeyMapError::NoKeysRemaining) => {
            if binding.expects.is_some() || binding.kind == BindingKind::Repeat {
                return Err(KeyMapError::KeySequenceIncomplete);
            } else {
                return Ok(binding);
            }
        }
        Err(error) => return Err(error),
    };

    let result = if binding.expects.is_some() {
        binding.kind = match combine(&binding, &next) {
            Ok(it) => it,
            Err(err) => {
                tracing::debug!("binding combine failed for {:?}: {:?}", binding, err);
                return Err(err);
            }
        };
        binding
    } else if let BindingKind::Repeat = binding.kind {
        next.repeat = get_repeat(&binding, &next);
        next
    } else {
        binding
    };

    Ok(result)
}

fn return_raw_if_expected(
    before: Option<&Binding>,
    keys: &[Key],
) -> Result<Option<Binding>, KeyMapError> {
    if let Some(before) = before {
        if let Some(message::NextBindingKind::Raw(regex)) = &before.expects {
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

            if let Some(regex) = regex {
                if !regex.is_match(&string) {
                    return Err(KeyMapError::NoValidBindingFound);
                }
            }

            return Ok(Some(Binding {
                kind: BindingKind::Raw(chars[0]),
                ..Default::default()
            }));
        }
    }

    Ok(None)
}

fn get_binding_by_keys(
    before: Option<&Binding>,
    tree: &KeyTree,
    mode: &Mode,
    keys: &[Key],
) -> Result<(Binding, Vec<Key>), KeyMapError> {
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

    Ok((binding, unused_keys))
}

fn combine(current: &Binding, next: &Binding) -> Result<BindingKind, KeyMapError> {
    match (&current.kind, &next.kind) {
        (BindingKind::Message(msg), BindingKind::Raw(raw)) => {
            let message = match msg {
                Message::PasteFromJunkYard(_) => Message::PasteFromJunkYard(*raw),
                Message::NavigateToMark(_) => Message::NavigateToMark(*raw),
                Message::ReplayMacro(_) => Message::ReplayMacro(*raw),
                Message::SetMark(_) => Message::SetMark(*raw),
                Message::StartMacro(_) => Message::StartMacro(*raw),
                _ => return Err(KeyMapError::NoValidBindingFound),
            };

            Ok(BindingKind::Message(message))
        }
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

fn get_messages_from_binding(mode: &Mode, binding: Binding) -> Vec<Message> {
    let mut messages = Vec::new();
    if let Some(md) = &binding.force {
        messages.push(Message::Buffer(BufferMessage::ChangeMode(
            mode.clone(),
            md.clone(),
        )));
    };

    let repeat = binding.repeat.unwrap_or(1);
    match &binding.kind {
        BindingKind::Message(msg) => messages.extend(get_repeated_message(repeat, msg)),
        BindingKind::Modification(mdf) => messages.push(Message::Buffer(
            BufferMessage::Modification(repeat, mdf.clone()),
        )),
        BindingKind::Motion(mtn) => messages.push(Message::Buffer(BufferMessage::MoveCursor(
            repeat,
            mtn.clone(),
        ))),
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
        Message::YankToJunkYard(_) => messages.push(Message::YankToJunkYard(repeat)),
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
        Mode::Command(_) | Mode::Insert => true,
        Mode::Navigation | Mode::Normal => false,
    }
}
