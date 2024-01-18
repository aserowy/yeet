use crate::key::{KeyCode, Key};

#[derive(Debug, Default)]
pub struct KeyBuffer {
    buffer: Vec<Key>,
}

impl KeyBuffer {
    pub fn add_key(&mut self, key: Key) {
        if key.code == KeyCode::Esc && !self.buffer.is_empty() {
            self.buffer.clear();
            return;
        }

        self.buffer.push(key);
    }

    pub fn get_keys(&self) -> Vec<Key> {
        self.buffer.to_vec()
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

impl ToString for KeyBuffer {
    fn to_string(&self) -> String {
        todo!()
    }
}
