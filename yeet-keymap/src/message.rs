use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub struct Binding {
    pub expects: Option<NextBindingKind>,
    pub force: Option<Mode>,
    pub kind: BindingKind,
    pub repeatable: bool,
}

impl Default for Binding {
    fn default() -> Self {
        Self {
            expects: None,
            force: None,
            kind: BindingKind::default(),
            repeatable: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum NextBindingKind {
    Raw,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum BindingKind {
    Message(Message),
    Motion(CursorDirection),
    #[default]
    None,
    Raw(char),
    Repeat(usize),
    RepeatOrMotion(usize, CursorDirection),
    Modification(TextModification),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Message {
    Buffer(Buffer),
    EnumerationChanged(PathBuf, Vec<(ContentKind, String)>),
    EnumerationFinished(PathBuf),
    Error(String),
    ExecuteCommand,
    ExecuteCommandString(String),
    KeySequenceChanged(String),
    NavigateToParent,
    NavigateToPath(PathBuf),
    NavigateToSelected,
    OpenSelected,
    PasteRegister(String),
    PathRemoved(PathBuf),
    PathsAdded(Vec<PathBuf>),
    PathsWriteFinished(Vec<PathBuf>),
    PreviewLoaded(PathBuf, Vec<String>),
    Print(Vec<PrintContent>),
    Rerender,
    Resize(u16, u16),
    Quit,
    YankSelected(usize),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Buffer {
    ChangeMode(Mode, Mode),
    Modification(usize, TextModification),
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CursorDirection {
    Bottom,
    Down,
    FindForward(char),
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContentKind {
    Directory,
    File,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ViewPortDirection {
    BottomOnCursor,
    CenterOnCursor,
    HalfPageDown,
    HalfPageUp,
    TopOnCursor,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrintContent {
    Error(String),
    Info(String),
}
