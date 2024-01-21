use std::hash::Hash;

#[derive(Clone, Debug, Eq)]
pub struct Key {
    pub code: KeyCode,
    pub modifiers: Vec<KeyModifier>,
}

impl Key {
    pub fn new(key: KeyCode, modifiers: Vec<KeyModifier>) -> Self {
        Self {
            code: key,
            modifiers,
        }
    }
}

impl Hash for Key {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to_string().hash(state);
    }
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        if self.code != other.code {
            return false;
        }

        if self.modifiers.len() != other.modifiers.len() {
            return false;
        }

        for modifier in &self.modifiers {
            if !other.modifiers.contains(modifier) {
                return false;
            }
        }

        true
    }
}

impl ToString for Key {
    fn to_string(&self) -> String {
        let mut modifiers = self.modifiers.clone();
        modifiers.sort();

        match self.code {
            KeyCode::Char(_) => {
                if modifiers.is_empty() {
                    self.code.to_string()
                } else if modifiers.len() == 1 && modifiers[0] == KeyModifier::Shift {
                    self.code.to_string().to_uppercase()
                } else if modifiers.contains(&KeyModifier::Shift) {
                    modifiers.retain(|modifier| *modifier != KeyModifier::Shift);
                    get_key_string(self.code.to_string().to_uppercase(), modifiers)
                } else {
                    get_key_string(self.code.to_string(), modifiers)
                }
            }
            _ => get_key_string(self.code.to_string(), modifiers),
        }
    }
}

fn get_key_string(code: String, modifiers: Vec<KeyModifier>) -> String {
    let mut result = String::from("<");

    for modifier in modifiers {
        match modifier {
            KeyModifier::Alt => result.push_str("A-"),
            KeyModifier::Command => result.push_str("D-"),
            KeyModifier::Ctrl => result.push_str("C-"),
            KeyModifier::Shift => result.push_str("S-"),
        };
    }

    result.push_str(&code);
    result.push('>');

    result
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum KeyCode {
    Backslash,
    Backspace,
    Bar,
    Char(char),
    Delete,
    Down,
    End,
    Enter,
    Esc,
    F(u8),
    Home,
    Help,
    Insert,
    Left,
    LessThan,
    Null,
    PageDown,
    PageUp,
    Print,
    Right,
    Space,
    Tab,
    Undo,
    Up,
}

impl ToString for KeyCode {
    fn to_string(&self) -> String {
        match self {
            KeyCode::Backslash => String::from("bslash"),
            KeyCode::Backspace => String::from("bs"),
            KeyCode::Bar => String::from("bar"),
            KeyCode::Char(c) => c.to_string().to_lowercase(),
            KeyCode::Delete => String::from("del"),
            KeyCode::Down => String::from("down"),
            KeyCode::End => String::from("end"),
            KeyCode::Enter => String::from("cr"),
            KeyCode::Esc => String::from("esc"),
            KeyCode::F(n) => format!("f{}", n),
            KeyCode::Home => String::from("home"),
            KeyCode::Help => String::from("help"),
            KeyCode::Insert => String::from("insert"),
            KeyCode::Left => String::from("left"),
            KeyCode::LessThan => String::from("lt"),
            KeyCode::Null => String::from("nul"),
            KeyCode::PageDown => String::from("pagedown"),
            KeyCode::PageUp => String::from("pageup"),
            KeyCode::Print => String::from("print"),
            KeyCode::Right => String::from("right"),
            KeyCode::Space => String::from("space"),
            KeyCode::Tab => String::from("tab"),
            KeyCode::Undo => String::from("undo"),
            KeyCode::Up => String::from("up"),
        }
    }
}

impl KeyCode {
    pub fn from_char(c: char) -> KeyCode {
        match c {
            '\\' => KeyCode::Backslash,
            '|' => KeyCode::Bar,
            '<' => KeyCode::LessThan,
            ' ' => KeyCode::Space,
            passed => KeyCode::Char(passed.to_ascii_lowercase()),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum KeyModifier {
    Alt,
    Command,
    Ctrl,
    Shift,
}
