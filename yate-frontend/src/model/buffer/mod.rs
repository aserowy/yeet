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

#[derive(Debug, Default)]
pub struct Cursor {
    pub hide_cursor: bool,
    pub hide_cursor_line: bool,
    pub horizontial_index: CursorPosition,
    pub vertical_index: usize,
}

#[derive(Debug, PartialEq)]
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
    pub style: Vec<StylePartialSpan>,
}

impl BufferLine {
    pub fn len(&self) -> usize {
        self.content.chars().count()
    }
}

pub type StylePartialSpan = (usize, usize, StylePartial);

#[derive(Clone, Debug)]
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
