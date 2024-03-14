use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub struct Binding {
    pub expects: Option<NextBindingKind>,
    pub force: Option<Mode>,
    pub kind: BindingKind,
    pub repeat: Option<usize>,
    pub repeatable: bool,
}

impl Default for Binding {
    fn default() -> Self {
        Self {
            expects: None,
            force: None,
            kind: BindingKind::default(),
            repeat: None,
            repeatable: true,
        }
    }
}

impl Binding {
    pub fn from_motion(motion: CursorDirection) -> Self {
        Self {
            kind: BindingKind::Motion(motion),
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum NextBindingKind {
    Motion,
    Raw,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum BindingKind {
    Message(Message),
    Motion(CursorDirection),
    #[default]
    None,
    Raw(char),
    Repeat,
    RepeatOrMotion(CursorDirection),
    Modification(TextModification),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Message {
    Buffer(Buffer),
    DeleteMarks(Vec<char>),
    ClearSearchHighlight,
    EnumerationChanged(PathBuf, Vec<(ContentKind, String)>, Option<String>),
    EnumerationFinished(PathBuf, Option<String>),
    Error(String),
    ExecuteCommand,
    ExecuteCommandString(String),
    KeySequenceChanged(String),
    NavigateToMark(char),
    NavigateToParent,
    NavigateToPath(PathBuf),
    NavigateToPathAsPreview(PathBuf),
    NavigateToSelected,
    OpenSelected,
    PasteFromJunkYard(String),
    PathRemoved(PathBuf),
    PathsAdded(Vec<PathBuf>),
    PreviewLoaded(PathBuf, Vec<String>),
    Print(Vec<PrintContent>),
    Rerender,
    Resize(u16, u16),
    SetMark(char),
    ToggleQuickFix,
    Quit,
    // TODO: yank to junk with motion
    YankToJunkYard(usize),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Buffer {
    // TODO: Yank & Paste in normal mode into reg (not freg!)
    ChangeMode(Mode, Mode),
    Modification(usize, TextModification),
    MoveCursor(usize, CursorDirection),
    MoveViewPort(ViewPortDirection),
    SaveBuffer(Option<usize>),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TextModification {
    DeleteLine,
    DeleteMotion(usize, CursorDirection),
    Insert(String),
    InsertNewLine(LineDirection),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum LineDirection {
    Up,
    Down,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum CursorDirection {
    Bottom,
    Down,
    FindBackward(char),
    FindForward(char),
    Left,
    LineEnd,
    LineStart,
    Right,
    Search(bool),
    TillBackward(char),
    TillForward(char),
    Top,
    Up,
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub enum Mode {
    Command(CommandMode),
    Insert,
    #[default]
    Navigation,
    Normal,
}

impl Mode {
    pub fn is_command(&self) -> bool {
        matches!(self, Mode::Command(_))
    }
}

impl ToString for Mode {
    fn to_string(&self) -> String {
        match self {
            Mode::Command(_) => "command".to_string(),
            Mode::Insert => "insert".to_string(),
            Mode::Navigation => "navigation".to_string(),
            Mode::Normal => "normal".to_string(),
        }
    }
}
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum CommandMode {
    Command,
    Search(SearchDirection),
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub enum SearchDirection {
    Up,
    #[default]
    Down,
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
    Default(String),
    Information(String),
}
