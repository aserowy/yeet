use action::{Action, Mode};
use buffer::KeyBuffer;
use key::Key;
use map::KeyMap;
use tree::{KeyTree, Node};

pub mod action;
mod buffer;
pub mod conversion;
pub mod key;
mod map;
mod tree;

#[derive(Debug)]
pub struct ActionResolver {
    buffer: KeyBuffer,
    tree: KeyTree,
    pub mode: Mode,
}

impl Default for ActionResolver {
    fn default() -> Self {
        Self {
            buffer: KeyBuffer::default(),
            tree: KeyMap::default().into_tree(),
            mode: Mode::default(),
        }
    }
}

impl ActionResolver {
    pub fn add_and_resolve(&mut self, key: Key) -> Option<Action> {
        self.buffer.add_key(key);

        let keys = self.buffer.get_keys();
        let node = self.tree.get_node(&self.mode, &keys);

        match node {
            Some(nd) => match nd {
                Node::Key(_) => None,
                Node::Action(action) => {
                    self.buffer.clear();

                    if let Action::ChangeMode(mode) = &action {
                        self.mode = mode.clone();
                    }

                    Some(action)
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
