use crate::{KeyPress, KeyCode, KeyModifier};

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
        self.buffer
            .iter()
            .map(|keypress| match keypress {
                KeyPress::Key(keycode) => match keycode {
                    KeyCode::Char(c) => c.to_string(),
                    KeyCode::Bar => "|".to_string(),
                    KeyCode::LessThan => "<".to_string(),
                    KeyCode::Space => " ".to_string(),
                    KeyCode::Esc => "Esc".to_string(),
                    KeyCode::Enter => "Enter".to_string(),
                    KeyCode::Backspace => "Backspace".to_string(),
                    KeyCode::Delete => "Delete".to_string(),
                    KeyCode::Down => "Down".to_string(),
                    KeyCode::End => "End".to_string(),
                    KeyCode::F(n) => format!("F{}", n),
                    KeyCode::Home => "Home".to_string(),
                    KeyCode::Insert => "Insert".to_string(),
                    KeyCode::Left => "Left".to_string(),
                    KeyCode::Null => "Null".to_string(),
                    KeyCode::PageDown => "PageDown".to_string(),
                    KeyCode::PageUp => "PageUp".to_string(),
                    KeyCode::Print => "Print".to_string(),
                    KeyCode::Right => "Right".to_string(),
                    KeyCode::Tab => "Tab".to_string(),
                    KeyCode::Undo => "Undo".to_string(),
                    KeyCode::Up => "Up".to_string(),
                    KeyCode::Help => todo!(),
                },
                KeyPress::Modifier(modifier, active) => match modifier {
                    KeyModifier::Alt => format!("Alt{}", if *active { "+" } else { "-" }),
                    KeyModifier::Command => format!("Command{}", if *active { "+" } else { "-" }),
                    KeyModifier::Ctrl => format!("Ctrl{}", if *active { "+" } else { "-" }),
                    KeyModifier::Shift => format!("Shift{}", if *active { "+" } else { "-" }),
                },
            })
            .collect::<Vec<String>>()
            .join(" ")
    }
}

