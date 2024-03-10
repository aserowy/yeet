use ratatui::style::{Color, Modifier};

use crate::settings::{self};

use self::{
    undo::{BufferChanged, Undo},
    viewport::ViewPort,
};

pub mod undo;
pub mod viewport;

#[derive(Debug, Default)]
pub struct Buffer {
    pub cursor: Option<Cursor>,
    pub lines: Vec<BufferLine>,
    pub show_border: bool,
    pub undo: Undo,
    pub view_port: ViewPort,
}

impl Buffer {
    pub fn set(&mut self, settings: &settings::Buffer) {
        self.view_port.sign_column_width = settings.sign_column_width;
    }
}

#[derive(Clone, Debug, Default)]
pub struct Cursor {
    pub hide_cursor: bool,
    pub hide_cursor_line: bool,
    pub horizontal_index: CursorPosition,
    pub vertical_index: usize,
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, Default)]
pub struct BufferLine {
    pub prefix: Option<String>,
    pub content: String,
    pub search: Option<Vec<StylePartialSpan>>,
    pub signs: Vec<Sign>,
    pub style: Vec<StylePartialSpan>,
}

impl BufferLine {
    pub fn len(&self) -> usize {
        self.content.chars().count()
    }
}

#[derive(Clone, Debug)]
pub struct Sign {
    pub id: SignIdentifier,
    pub content: char,
    pub priority: usize,
    pub style: Vec<StylePartial>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SignIdentifier {
    Mark,
    QuickFix,
}

#[derive(Clone, Debug, Default)]
pub struct StylePartialSpan {
    pub start: usize,
    pub end: usize,
    pub style: StylePartial,
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub enum BufferResult {
    Changes(Vec<BufferChanged>),
    _Unused,
}
