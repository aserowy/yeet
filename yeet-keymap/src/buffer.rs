use crate::key::{Key, KeyCode};

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

    pub fn to_keycode_string(&self) -> String {
        let mut result = String::new();
        for key in &self.buffer {
            result.push_str(&key.to_keycode_string());
        }

        result
    }
}

impl ToString for KeyBuffer {
    fn to_string(&self) -> String {
        let mut result = String::new();
        for key in &self.buffer {
            result.push_str(&key.to_string());
        }

        result
    }
}
