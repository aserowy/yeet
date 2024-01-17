use crate::{KeyPress, KeyCode};

#[derive(Default)]
pub struct KeyBuffer {
    buffer: Vec<KeyPress>,
}

impl KeyBuffer {
    pub fn add_keypress(&mut self, keypress: KeyPress) {
        if keypress == KeyPress::Key(KeyCode::Esc) && !self.buffer.is_empty() {
            self.buffer.clear();
            return;
        }

        self.buffer.push(keypress);
    }
}

impl ToString for KeyBuffer {
    fn to_string(&self) -> String {
        todo!()
    }
}

