#[derive(Clone, Debug, PartialEq)]
pub struct Key {
    pub code: KeyCode,
    pub modifiers: Vec<KeyModifier>,
}

impl Key {
    pub fn new(key: KeyCode, modifiers: Vec<KeyModifier>) -> Self {
        Self { code: key, modifiers }
    }
}

// impl ToString for KeyStroke {
//     fn to_string(&self) -> String {
//     }
// }

#[derive(Clone, Debug, PartialEq)]
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
    pub fn from_char(c: char) -> KeyCode {
        match c {
            '\\' => KeyCode::Backslash,
            '|' => KeyCode::Bar,
            '<' => KeyCode::LessThan,
            ' ' => KeyCode::Space,
            passed => KeyCode::Char(passed),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum KeyModifier {
    Alt,
    Command,
    Ctrl,
    Shift,
}
