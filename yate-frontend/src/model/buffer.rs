#[derive(Debug, Default)]
pub struct Buffer {
    pub cursor: Option<Cursor>,
    pub lines: Vec<String>,
    pub view_port: ViewPort,
}

#[derive(Debug, Default)]
pub struct Cursor {
    pub horizontial_index: CursorPosition,
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

#[derive(Debug, Default)]
pub struct ViewPort {
    pub content_width: usize,
    pub height: usize,
    pub horizontal_index: usize,
    pub line_number: LineNumber,
    pub line_number_width: usize,
    pub vertical_index: usize,
}

impl ViewPort {
    pub fn get_offset_width(&self) -> usize {
        self.get_line_number_width() + 1
    }

    pub fn get_line_number_width(&self) -> usize {
        match self.line_number {
            LineNumber::_Absolute => self.line_number_width,
            LineNumber::None => 0,
            LineNumber::Relative => self.line_number_width,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum LineNumber {
    _Absolute,

    #[default]
    None,

    Relative,
}
