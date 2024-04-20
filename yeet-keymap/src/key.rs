use std::{collections::VecDeque, hash::Hash};

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

    pub fn to_keycode_string(&self) -> String {
        let mut modifiers = self.modifiers.clone();
        modifiers.sort();

        match self.code {
            KeyCode::Char(_) => {
                if modifiers.contains(&KeyModifier::Shift) {
                    modifiers.retain(|modifier| *modifier != KeyModifier::Shift);
                    get_key_string(self.code.to_string().to_uppercase(), modifiers, false)
                } else {
                    get_key_string(self.code.to_string(), modifiers, false)
                }
            }
            _ => get_key_string(self.code.to_string(), modifiers, true),
        }
    }

    pub fn from_keycode_string(keycode: &str) -> Option<Self> {
        let regex = regex::Regex::new(r"[^-<>]+|^-$|--").expect("Failed to compile regex");
        let mut codes = regex
            .find_iter(keycode)
            .map(|m| m.as_str())
            .collect::<VecDeque<_>>();

        let mut modifiers = Vec::new();
        let mut last = codes.pop_back()?;
        if last == "--" {
            last = "-";
        }

        if last.chars().count() == 1 && last.chars().last()?.is_ascii_uppercase() {
            modifiers.push(KeyModifier::Shift);
        }

        for modifier in codes {
            match modifier.to_ascii_uppercase().as_str() {
                "A" => modifiers.push(KeyModifier::Alt),
                "C" => modifiers.push(KeyModifier::Ctrl),
                "D" => modifiers.push(KeyModifier::Command),
                "S" => modifiers.push(KeyModifier::Shift),
                _ => (),
            }
        }

        KeyCode::from_keycode_string(last).map(|code| Self { code, modifiers })
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
                if modifiers.contains(&KeyModifier::Shift) {
                    modifiers.retain(|modifier| *modifier != KeyModifier::Shift);
                    get_key_string(self.code.to_string().to_uppercase(), modifiers, false)
                } else {
                    get_key_string(self.code.to_string(), modifiers, false)
                }
            }
            KeyCode::Bar => get_key_string(String::from("|"), modifiers, false),
            KeyCode::Backslash => get_key_string(String::from("\\"), modifiers, false),
            KeyCode::LessThan => get_key_string(String::from("<"), modifiers, false),
            KeyCode::Space => get_key_string(String::from(" "), modifiers, false),
            KeyCode::Tab => get_key_string(String::from("\\t"), modifiers, false),
            _ => get_key_string(self.code.to_string(), modifiers, true),
        }
    }
}

fn get_key_string(code: String, modifiers: Vec<KeyModifier>, force_ltgt: bool) -> String {
    if modifiers.is_empty() && !force_ltgt {
        return code;
    }

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

impl KeyCode {
    pub fn from_keycode_string(keycode: &str) -> Option<Self> {
        match keycode {
            "bslash" => Some(KeyCode::Backslash),
            "bs" => Some(KeyCode::Backspace),
            "bar" => Some(KeyCode::Bar),
            "del" => Some(KeyCode::Delete),
            "down" => Some(KeyCode::Down),
            "end" => Some(KeyCode::End),
            "cr" => Some(KeyCode::Enter),
            "esc" => Some(KeyCode::Esc),
            "home" => Some(KeyCode::Home),
            "help" => Some(KeyCode::Help),
            "insert" => Some(KeyCode::Insert),
            "left" => Some(KeyCode::Left),
            "lt" => Some(KeyCode::LessThan),
            "nul" => Some(KeyCode::Null),
            "pagedown" => Some(KeyCode::PageDown),
            "pageup" => Some(KeyCode::PageUp),
            "print" => Some(KeyCode::Print),
            "right" => Some(KeyCode::Right),
            "space" => Some(KeyCode::Space),
            "tab" => Some(KeyCode::Tab),
            "undo" => Some(KeyCode::Undo),
            "up" => Some(KeyCode::Up),
            code => {
                if code.len() == 1 {
                    code.chars().next().map(Self::from_char)
                } else {
                    None
                }
            }
        }
    }

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

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum KeyModifier {
    Alt,
    Command,
    Ctrl,
    Shift,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_keycode_string_valid() {
        let keycode = "a";
        let result = Key::from_keycode_string(keycode);
        assert!(result.is_some(), "Expected Some(Key), got None");
    }

    #[test]
    fn from_keycode_string_dash() {
        let keycode = "-";
        let result = Key::from_keycode_string(keycode);
        assert!(result.is_some(), "Expected Some(Key), got None");
    }

    #[test]
    fn from_keycode_string_invalid() {
        let keycode = "<>";
        let result = Key::from_keycode_string(keycode);
        assert!(result.is_none(), "Expected None, got Some(Key)");
    }

    #[test]
    fn from_keycode_string_empty() {
        let keycode = "";
        let result = Key::from_keycode_string(keycode);
        assert!(result.is_none(), "Expected None, got Some(Key)");
    }

    #[test]
    fn from_keycode_string_case_sensitive() {
        let keycode = "A";
        let result = Key::from_keycode_string(keycode);
        assert!(result.is_some(), "Expected Some(Key), got None");
        assert_eq!(result.as_ref().unwrap().code, KeyCode::Char('a'));
        assert_eq!(result.unwrap().modifiers, vec![KeyModifier::Shift]);
    }

    #[test]
    fn from_keycode_string_with_modifiers() {
        let keycode = "<A-C-lt>";
        let result = Key::from_keycode_string(keycode);
        assert!(result.is_some(), "Expected Some(Key), got None");
        assert_eq!(result.as_ref().unwrap().code, KeyCode::LessThan);

        let modifiers = result.unwrap().modifiers;
        assert!(modifiers.contains(&KeyModifier::Alt));
        assert!(modifiers.contains(&KeyModifier::Ctrl));
    }

    #[test]
    fn from_keycode_string_dash_with_modifiers() {
        let keycode = "<A-C-->";
        let result = Key::from_keycode_string(keycode);
        assert!(result.is_some(), "Expected Some(Key), got None");
        assert_eq!(result.as_ref().unwrap().code, KeyCode::from_char('-'));

        let modifiers = result.unwrap().modifiers;
        assert!(modifiers.contains(&KeyModifier::Alt));
        assert!(modifiers.contains(&KeyModifier::Ctrl));
    }

    #[test]
    fn from_keycode_string_with_invalid_modifiers() {
        let keycode = "<A-C-invalid>";
        let result = Key::from_keycode_string(keycode);
        assert!(result.is_none(), "Expected None, got Some(Key)");
    }
}
