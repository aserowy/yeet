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
    // TODO: none
}

impl Default for CursorPosition {
    fn default() -> Self {
        CursorPosition::Absolute(0)
    }
}

#[derive(Debug, Default)]
pub struct ViewPort {
    pub horizontal_index: usize,
    pub vertical_index: usize,
    pub height: usize,
    pub width: usize,
}
