use ratatui::style::{Color, Modifier};

#[derive(Debug, Default)]
pub struct Buffer {
    pub cursor: Option<Cursor>,
    pub lines: Vec<BufferLine>,
    pub view_port: ViewPort,
}

#[derive(Debug, Default)]
pub struct Cursor {
    pub horizontial_index: CursorPosition,
    pub hide_cursor_line: bool,
    pub vertical_index: usize,
}

#[derive(Debug)]
pub enum CursorPosition {
    Absolute(usize),
    End,
    None,
}

impl Default for CursorPosition {
    fn default() -> Self {
        CursorPosition::Absolute(0)
    }
}

#[derive(Clone, Debug, Default)]
pub struct BufferLine {
    pub content: String,
    pub style: Vec<StylePartialSpan>,
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

#[derive(Debug, Default)]
pub struct ViewPort {
    pub height: usize,
    pub horizontal_index: usize,
    pub line_number: LineNumber,
    pub line_number_width: usize,
    pub vertical_index: usize,
    pub width: usize,
}

impl ViewPort {
    pub fn get_border_width(&self) -> usize {
        if self.get_prefix_width() > 0 {
            1
        } else {
            0
        }
    }

    pub fn get_content_width(&self) -> usize {
        self.width - self.get_offset_width()
    }

    pub fn get_line_number_width(&self) -> usize {
        match self.line_number {
            LineNumber::_Absolute => self.line_number_width,
            LineNumber::None => 0,
            LineNumber::Relative => self.line_number_width,
        }
    }

    pub fn get_offset_width(&self) -> usize {
        self.get_line_number_width() + self.get_border_width()
    }

    pub fn get_prefix_width(&self) -> usize {
        self.get_line_number_width()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum LineNumber {
    _Absolute,

    #[default]
    None,

    Relative,
}
