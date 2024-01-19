#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    KeySequenceChanged(String),
    ModeChanged(Mode),
    MoveCursorDown,
    Refresh,
    Quit,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Mode {
    Normal,
    Command,
}

impl ToString for Mode {
    fn to_string(&self) -> String {
        match self {
            Mode::Normal => format!("normal"),
            Mode::Command => format!("command"),
        }
    }
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Normal
    }
}
