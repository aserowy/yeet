use super::BufferLine;

#[derive(Debug, Default)]
pub struct ViewPort {
    pub height: usize,
    pub horizontal_index: usize,
    pub line_number: LineNumber,
    pub line_number_width: usize,
    pub sign_column_width: usize,
    pub vertical_index: usize,
    pub width: usize,
}

// TODO: enable with settings
// TODO: refactor into functions
impl ViewPort {
    pub fn get_border_width(&self) -> usize {
        if self.get_prefix_width() > 0 {
            1
        } else {
            0
        }
    }

    pub fn get_content_width(&self, line: &BufferLine) -> usize {
        self.width - self.get_offset_width(line)
    }

    pub fn get_line_number_width(&self) -> usize {
        match self.line_number {
            LineNumber::_Absolute => self.line_number_width,
            LineNumber::None => 0,
            LineNumber::Relative => self.line_number_width,
        }
    }

    pub fn get_offset_width(&self, line: &BufferLine) -> usize {
        let custom_prefix_width = if let Some(prefix) = &line.prefix {
            prefix.chars().count()
        } else {
            0
        };

        self.get_prefix_width() + self.get_border_width() + custom_prefix_width
    }

    fn get_prefix_width(&self) -> usize {
        self.sign_column_width + self.get_line_number_width()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum LineNumber {
    _Absolute,
    #[default]
    None,
    Relative,
}
