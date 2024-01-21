use buffer::KeyBuffer;
use key::Key;
use map::KeyMap;
use message::{Message, Mode};
use tree::{KeyTree, Node};

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
    pub fn add_and_resolve(&mut self, key: Key) -> Option<Vec<Message>> {
        self.buffer.add_key(key);

        let keys = self.buffer.get_keys();
        let node = self.tree.get_node(&self.mode, &keys);

        match node {
            Some(nd) => match nd {
                Node::Key(_) => None,
                Node::Message(message) => {
                    self.buffer.clear();

                    if let Message::ChangeMode(mode) = &message {
                        self.mode = mode.clone();
                    }

                    Some(vec![message])
                }
            },
            None => {
                self.buffer.clear();
                None
            }
        }
    }

    pub fn get_key_string(&self) -> String {
        self.buffer.to_string()
    }
}
