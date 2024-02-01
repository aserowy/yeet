#[derive(Clone, Debug, PartialEq)]
pub enum Binding {
    Message(Message),
    Mode(Mode),
    Motion(CursorDirection),
    Repeat(usize),
    RepeatOrMotion(usize, CursorDirection),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    ChangeKeySequence(String),
    ChangeMode(Mode, Mode),
    ExecuteCommand,
    Modification(TextModification),
    MoveCursor(usize, CursorDirection),
    MoveViewPort(ViewPortDirection),
    SelectCurrent,
    SelectParent,
    Refresh,
    Quit,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TextModification {
    DeleteCharBeforeCursor,
    DeleteCharOnCursor,
    DeleteLineOnCursor,
    Insert(String),
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

#[derive(Clone, Debug, PartialEq)]
pub enum ViewPortDirection {
    BottomOnCursor,
    CenterOnCursor,
    HalfPageDown,
    HalfPageUp,
    TopOnCursor,
}
