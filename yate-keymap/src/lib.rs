use buffer::KeyBuffer;
use key::{Key, KeyCode};
use map::KeyMap;
use message::{Binding, Message, Mode};
use tree::KeyTree;

mod buffer;
pub mod conversion;
pub mod key;
mod map;
pub mod message;
mod tree;

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
        let default_result = vec![Message::ChangeKeySequence(self.buffer.to_string())];

        let keys = self.buffer.get_keys();
        if &key.code == &KeyCode::Esc && !keys.is_empty() {
            self.buffer.clear();
            return default_result;
        }

        self.buffer.add_key(key);

        let keys = self.buffer.get_keys();
        let (bindings, node) = self.tree.get_bindings(&self.mode, &keys);
        match (bindings, node) {
            (_, Some(_)) => default_result,
            (bindings, None) => {
                if bindings.is_empty() {
                    self.buffer.clear();
                    return default_result;
                }

                let messages = get_messages_from_bindings(bindings, &mut self.mode);
                if messages.is_empty() {
                    default_result
                } else {
                    self.buffer.clear();
                    messages
                }
            }
        }
    }
}

fn get_messages_from_bindings(bindings: Vec<Binding>, mode: &mut Mode) -> Vec<Message> {
    let mut repeat = None;
    let mut messages = Vec::new();
    for binding in bindings {
        match binding {
            Binding::Message(msg) => {
                if let Message::ChangeMode(md) = &msg {
                    *mode = md.clone();
                }

                match repeat {
                    Some(rpt) => {
                        for _ in 0..rpt {
                            messages.push(msg.clone());
                        }
                        repeat = None;
                    }
                    None => messages.push(msg),
                }
            }
            Binding::Repeat(rpt) => match repeat {
                Some(r) => repeat = Some(r * 10 + rpt),
                None => repeat = Some(rpt),
            },
            Binding::Motion(mtn) => match repeat {
                Some(rpt) => {
                    messages.push(Message::MoveCursor(rpt, mtn));
                    repeat = None;
                }
                None => messages.push(Message::MoveCursor(1, mtn)),
            },
        }
    }

    messages
}
