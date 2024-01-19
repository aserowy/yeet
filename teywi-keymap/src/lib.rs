use action::{Action, Mode};
use buffer::KeyBuffer;
use key::Key;
use map::KeyMap;

pub mod action;
mod buffer;
pub mod conversion;
pub mod key;
mod map;

#[derive(Debug, Default)]
pub struct ActionResolver {
    buffer: KeyBuffer,
    map: KeyMap,
    pub mode: Mode,
}

impl ActionResolver {
    pub fn add_and_resolve(&mut self, key: Key) -> Option<Action> {
        self.buffer.add_key(key);

        let keys = self.buffer.get_keys();
        let action = self.map.get_action(&self.mode, &keys);

        if let Some(action) = action {
            self.buffer.clear();

            if let Action::ChangeMode(mode) = &action {
                self.mode = mode.clone();
            }

            return Some(action);
        }

        None
    }

    pub fn get_key_string(&self) -> String {
        self.buffer.to_string()
    }
}
