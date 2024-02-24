use buffer::KeyBuffer;
use key::{Key, KeyCode};
use map::KeyMap;
use message::{Binding, Message, Mode};
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
        let (bindings, node) = self.tree.get_bindings(&self.mode, &keys);
        let mut messages = match (bindings, node) {
            (_, Some(_)) => Vec::new(),
            (bindings, None) => {
                if bindings.is_empty() {
                    let messages = if get_passthrough_by_mode(&self.mode) {
                        let message = TextModification::Insert(self.buffer.to_string());
                        vec![Message::Buffer(Buffer::Modification(message))]
                    } else {
                        Vec::new()
                    };

                    self.buffer.clear();
                    messages
                } else {
                    let messages = get_messages_from_bindings(bindings, &mut self.mode);
                    if messages.is_empty() {
                        Vec::new()
                    } else {
                        self.buffer.clear();
                        messages
                    }
                }
            }
        };

        messages.push(Message::KeySequenceChanged(self.buffer.to_string()));
        messages
    }
}

fn get_messages_from_bindings(bindings: Vec<Binding>, mode: &mut Mode) -> Vec<Message> {
    let mut repeat = None;
    let mut messages = Vec::new();
    for binding in bindings {
        match binding {
            Binding::Message(msg) => match repeat {
                Some(rpt) => {
                    messages.extend(get_repeated_messages(&msg, rpt));
                    repeat = None;
                }
                None => messages.push(msg),
            },
            Binding::Mode(md) => {
                messages.push(Message::Buffer(Buffer::ChangeMode(
                    mode.clone(),
                    md.clone(),
                )));
            }
            Binding::ModeAndNotRepeatedMotion(md, mtn) => {
                messages.push(Message::Buffer(Buffer::ChangeMode(
                    mode.clone(),
                    md.clone(),
                )));
                messages.push(Message::Buffer(Buffer::MoveCursor(1, mtn)));

                repeat = None;
            }
            Binding::ModeAndTextModification(md, mdfctn) => {
                messages.push(Message::Buffer(Buffer::ChangeMode(
                    mode.clone(),
                    md.clone(),
                )));

                let msg = Message::Buffer(Buffer::Modification(mdfctn));
                match repeat {
                    Some(rpt) => {
                        messages.extend(get_repeated_messages(&msg, rpt));
                        repeat = None;
                    }
                    None => messages.push(msg),
                }
            }
            Binding::Motion(mtn) => match repeat {
                Some(rpt) => {
                    messages.push(Message::Buffer(Buffer::MoveCursor(rpt, mtn)));
                    repeat = None;
                }
                None => messages.push(Message::Buffer(Buffer::MoveCursor(1, mtn))),
            },
            Binding::Repeat(rpt) => match repeat {
                Some(r) => repeat = Some(r * 10 + rpt),
                None => repeat = Some(rpt),
            },
            Binding::RepeatOrMotion(rpt, mtn) => match repeat {
                Some(r) => repeat = Some(r * 10 + rpt),
                None => messages.push(Message::Buffer(Buffer::MoveCursor(1, mtn))),
            },
        }
    }

    messages
}

fn get_repeated_messages(msg: &Message, rpt: usize) -> Vec<Message> {
    let mut messages = Vec::new();
    match msg {
        Message::Buffer(Buffer::Modification(TextModification::DeleteLineOnCursor(_))) => messages
            .push(Message::Buffer(Buffer::Modification(
                TextModification::DeleteLineOnCursor(rpt),
            ))),
        Message::YankSelected(_) => messages.push(Message::YankSelected(rpt)),
        _ => {
            for _ in 0..rpt {
                messages.push(msg.clone());
            }
        }
    }
    messages
}

fn get_passthrough_by_mode(mode: &Mode) -> bool {
    match mode {
        Mode::Command => true,
        Mode::Insert => true,
        Mode::Navigation => false,
        Mode::Normal => false,
    }
}
