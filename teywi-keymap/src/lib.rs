pub mod buffer;
pub mod conversion;

#[derive(Clone, Debug)]
pub enum Action {
    Mode(Mode),
    NavigateUp,
    NavigateDown,
    NavigateParent,
    NavigateChild,
    Quit,
}

#[derive(Clone, Debug)]
pub enum Mode {
    Normal,
    Command,
}

#[derive(Clone, Debug, PartialEq)]
pub enum KeyPress {
    Key(KeyCode),
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

    pub fn to_string(&self, modifier: Vec<KeyModifier>) -> String {
        match self {
            KeyCode::Backslash => format!("<bslash>"),
            KeyCode::Backspace => format!("<bs>"),
            KeyCode::Bar => format!("<bar>"),
            KeyCode::Char(_) => todo!(),
            KeyCode::Delete => todo!(),
            KeyCode::Down => todo!(),
            KeyCode::End => todo!(),
            KeyCode::Enter => todo!(),
            KeyCode::Esc => todo!(),
            KeyCode::F(_) => todo!(),
            KeyCode::Home => todo!(),
            KeyCode::Help => todo!(),
            KeyCode::Insert => todo!(),
            KeyCode::Left => todo!(),
            KeyCode::LessThan => todo!(),
            KeyCode::Null => todo!(),
            KeyCode::PageDown => todo!(),
            KeyCode::PageUp => todo!(),
            KeyCode::Print => todo!(),
            KeyCode::Right => todo!(),
            KeyCode::Space => todo!(),
            KeyCode::Tab => todo!(),
            KeyCode::Undo => todo!(),
            KeyCode::Up => todo!(),
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

#[derive(Default)]
pub struct KeyMap {
    mappings: Vec<(String, Action)>,
}

impl KeyMap {}
