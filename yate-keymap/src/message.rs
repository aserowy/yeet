use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub enum Binding {
    Message(Message),
    Mode(Mode),
    ModeAndNotRepeatedMotion(Mode, CursorDirection),
    ModeAndTextModification(Mode, TextModification),
    Motion(CursorDirection),
    Repeat(usize),
    RepeatOrMotion(usize, CursorDirection),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    Buffer(Buffer),
    ChangeKeySequence(String),
    ExecuteCommand,
    PathsAdded(Vec<PathBuf>),
    PathRemoved(PathBuf),
    SelectCurrent,
    SelectParent,
    SelectPath(PathBuf),
    Quit,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Buffer {
    ChangeMode(Mode, Mode),
    Modification(TextModification),
    MoveCursor(usize, CursorDirection),
    MoveViewPort(ViewPortDirection),
    SaveBuffer(Option<usize>),
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
