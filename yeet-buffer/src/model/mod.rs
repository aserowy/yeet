use std::fmt::Display;

use ratatui::style::{Color, Modifier};

use crate::message::CursorDirection;

use self::{
    undo::{BufferChanged, Undo},
    viewport::ViewPort,
};

pub mod undo;
pub mod viewport;

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

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content = match self {
            Mode::Command(_) => "command".to_string(),
            Mode::Insert => "insert".to_string(),
            Mode::Navigation => "navigation".to_string(),
            Mode::Normal => "normal".to_string(),
        };

        write!(f, "{}", content)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum CommandMode {
    Command,
    PrintMultiline,
    Search(SearchDirection),
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub enum SearchDirection {
    #[default]
    Down,
    Up,
}

#[derive(Default)]
pub struct BufferSettings {
    pub sign_column_width: usize,
}

#[derive(Default)]
pub struct Buffer {
    pub cursor: Option<Cursor>,
    pub last_find: Option<CursorDirection>,
    pub lines: Vec<BufferLine>,
    pub show_border: bool,
    pub undo: Undo,
    pub view_port: ViewPort,
}

impl Buffer {
    pub fn set(&mut self, settings: &BufferSettings) {
        self.view_port.sign_column_width = settings.sign_column_width;
    }
}

#[derive(Clone, Default)]
pub struct Cursor {
    pub hide_cursor: bool,
    pub hide_cursor_line: bool,
    pub horizontal_index: CursorPosition,
    pub vertical_index: usize,
}

#[derive(Clone, PartialEq)]
pub enum CursorPosition {
    Absolute { current: usize, expanded: usize },
    End,
    None,
}

impl Default for CursorPosition {
    fn default() -> Self {
        CursorPosition::Absolute {
            current: 0,
            expanded: 0,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BufferLine {
    pub prefix: Option<String>,
    pub content: String,
    pub search: Option<Vec<StylePartialSpan>>,
    pub signs: Vec<Sign>,
    pub style: Vec<StylePartialSpan>,
}

impl BufferLine {
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn len(&self) -> usize {
        self.content.chars().count()
    }
}

pub type SignIdentifier = &'static str;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Sign {
    pub id: SignIdentifier,
    pub content: char,
    pub priority: usize,
    pub style: Vec<StylePartial>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StylePartialSpan {
    pub start: usize,
    pub end: usize,
    pub style: StylePartial,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StylePartial {
    Background(Color),
    Foreground(Color),
    Modifier(Modifier),
}

impl Default for StylePartial {
    fn default() -> Self {
        StylePartial::Foreground(Color::default())
    }
}

#[derive(Clone, PartialEq)]
pub enum BufferResult {
    Changes(Vec<BufferChanged>),
    CursorPositionChanged,
    FindScopeChanged(CursorDirection),
}
