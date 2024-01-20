#[derive(Debug, Default)]
pub struct Buffer {
    pub cursor: Cursor,
    pub lines: Vec<String>,
}

#[derive(Debug, Default)]
pub struct Cursor {
    pub horizontial_position: CursorPosition,
    pub line_number: usize,
}

#[derive(Debug)]
pub enum CursorPosition {
    Absolute(usize),
    End,
}

impl Default for CursorPosition {
    fn default() -> Self {
        CursorPosition::Absolute(0)
    }
}
