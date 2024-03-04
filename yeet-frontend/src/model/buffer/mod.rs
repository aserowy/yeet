use ratatui::style::{Color, Modifier};

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
    pub undo: Undo,
    pub view_port: ViewPort,
}

#[derive(Clone, Debug, Default)]
pub struct Cursor {
    pub hide_cursor: bool,
    pub hide_cursor_line: bool,
    pub horizontial_index: CursorPosition,
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
    pub style: Vec<StylePartialSpan>,
}

impl BufferLine {
    pub fn len(&self) -> usize {
        self.content.chars().count()
    }
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
        StylePartial::Foreground(Color::White)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BufferResult {
    Changes(Vec<BufferChanged>),
    _Unused,
}
