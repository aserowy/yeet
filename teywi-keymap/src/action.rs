#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    ChangeKeySequence(String),
    ChangeMode(Mode),
    MoveCursor(Direction),
    Refresh,
    Quit,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Direction {
    Bottom,
    Down,
    Left,
    LineEnd,
    LineStart,
    Right,
    Top,
    Up,
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
