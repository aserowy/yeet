#[derive(Debug, Default)]
pub struct Buffer {
    pub cursor: Cursor,
    pub lines: Vec<String>,
    pub view_port: ViewPort,
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

#[derive(Debug, Default)]
pub struct ViewPort {
    pub x: usize,
    pub y: usize,
    pub height: usize,
    pub width: usize,
}
