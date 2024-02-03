#[derive(Clone, Debug, PartialEq)]
pub enum Binding {
    Message(Message),
    Mode(Mode),
    ModeAndTextModification(Mode, TextModification),
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

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TextModification {
    DeleteCharBeforeCursor,
    DeleteCharOnCursor,
    DeleteLineOnCursor,
    Insert(String),
    InsertNewLine(NewLineDirection),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum NewLineDirection {
    Above,
    Under,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CursorDirection {
    Bottom,
    Down,
    Left,
    LineEnd,
    LineStart,
    Right,
    Validate,
    Top,
    Up,
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub enum Mode {
    Command,
    Insert,

    #[default]
    Navigation,

    Normal,
}

impl ToString for Mode {
    fn to_string(&self) -> String {
        match self {
            Mode::Command => "command".to_string(),
            Mode::Insert => "insert".to_string(),
            Mode::Navigation => "navigation".to_string(),
            Mode::Normal => "normal".to_string(),
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
