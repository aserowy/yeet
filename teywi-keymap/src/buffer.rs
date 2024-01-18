use crate::{
    key::{Key, KeyCode},
    map::KeyStroke,
};

#[derive(Debug, Default)]
pub struct KeyBuffer {
    buffer: Vec<Key>,
}

impl KeyBuffer {
    pub fn add_key(&mut self, key: Key) {
        if key == Key::Code(KeyCode::Esc) && !self.buffer.is_empty() {
            self.buffer.clear();
            return;
        }

        self.buffer.push(key);
    }

    pub fn get_keystrokes(&self) -> Vec<KeyStroke> {
        let mut keystrokes = Vec::new();
        let mut active_modifiers = Vec::new();

        for key in &self.buffer {
            match key {
                Key::Code(code) => {
                    keystrokes.push(KeyStroke {
                        key: code.clone(),
                        modifiers: active_modifiers.clone(),
                    });
                }
                Key::Modifier(modifier, active) => {
                    if *active {
                        active_modifiers.push(modifier.clone());
                    } else {
                        active_modifiers.retain(|m| m != modifier);
                    }
                }
            }
        }

        keystrokes
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
