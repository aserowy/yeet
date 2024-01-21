#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    ChangeKeySequence(String),
    ChangeMode(Mode),
    MoveCursor(CursorDirection),
    SelectCurrent,
    SelectParent,
    Refresh,
    Quit,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CursorDirection {
    Bottom,
    Down,
    Left,
    LineEnd,
    LineStart,
    Right,
    Top,
    Up,
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub enum Mode {
    #[default]
    Normal,
    Command,
}

impl ToString for Mode {
    fn to_string(&self) -> String {
        match self {
            Mode::Normal => "normal".to_string(),
            Mode::Command => "command".to_string(),
        }
    }
}
