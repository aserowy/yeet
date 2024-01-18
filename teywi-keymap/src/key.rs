#[derive(Clone, Debug, PartialEq)]
pub enum Key {
    Code(KeyCode),
    Modifier(KeyModifier, bool),
}

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
