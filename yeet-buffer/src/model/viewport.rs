use std::collections::HashSet;

use super::{BufferLine, Cursor, SignIdentifier};

#[derive(Debug, Default)]
pub struct WindowSettings {
    pub sign_column_width: usize,
}

#[derive(Clone, Debug, Default)]
pub struct ViewPort {
    pub buffer_id: usize,
    pub cursor: Cursor,
    pub hide_cursor: bool,
    pub hide_cursor_line: bool,
    pub height: u16,
    pub hidden_sign_ids: HashSet<SignIdentifier>,
    pub horizontal_index: usize,
    pub line_number: LineNumber,
    pub line_number_width: usize,
    pub prefix_column_width: usize,
    pub show_border: bool,
    pub sign_column_width: usize,
    pub vertical_index: usize,
    pub width: u16,
    pub wrap: bool,
    pub x: u16,
    pub y: u16,
}

impl ViewPort {
    pub fn get_border_width(&self) -> usize {
        if self.get_prefix_width() > 0 {
            1
        } else {
            0
        }
    }

    pub fn get_content_width(&self, line: &BufferLine) -> usize {
        usize::from(self.width)
            .saturating_sub(self.get_offset_width(line))
            .saturating_sub(self.get_border_width())
    }

    pub fn get_line_number_width(&self) -> usize {
        match self.line_number {
            LineNumber::Absolute => self.line_number_width,
            LineNumber::None => 0,
            LineNumber::Relative => self.line_number_width,
        }
    }

    pub fn get_offset_width(&self, line: &BufferLine) -> usize {
        self.get_prefix_width() + self.get_border_width() + self.get_custom_prefix_width(line)
    }

    fn get_custom_prefix_width(&self, line: &BufferLine) -> usize {
        if self.prefix_column_width > 0 {
            self.prefix_column_width
        } else if let Some(prefix) = &line.prefix {
            prefix.chars().count()
        } else {
            0
        }
    }

    fn get_prefix_width(&self) -> usize {
        self.sign_column_width + self.get_line_number_width()
    }

    pub fn set(&mut self, settings: &WindowSettings) {
        self.sign_column_width = settings.sign_column_width;
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum LineNumber {
    Absolute,
    #[default]
    None,
    Relative,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_column_width_defaults_to_zero() {
        let vp = ViewPort::default();
        assert_eq!(vp.prefix_column_width, 0);
    }

    #[test]
    fn prefix_width_excludes_prefix_column_when_zero() {
        let vp = ViewPort {
            sign_column_width: 2,
            line_number: LineNumber::Absolute,
            line_number_width: 3,
            prefix_column_width: 0,
            ..Default::default()
        };
        // prefix = sign(2) + line_number(3) = 5
        assert_eq!(vp.get_prefix_width(), 5);
    }

    #[test]
    fn offset_width_includes_prefix_column_when_set() {
        let vp = ViewPort {
            sign_column_width: 2,
            line_number: LineNumber::Absolute,
            line_number_width: 3,
            prefix_column_width: 2,
            ..Default::default()
        };
        let bl = BufferLine::default();
        // prefix = sign(2) + line_number(3) = 5, border = 1, custom_prefix = 2
        // offset = 5 + 1 + 2 = 8
        assert_eq!(vp.get_offset_width(&bl), 8);
    }

    #[test]
    fn offset_width_includes_prefix_column() {
        let vp = ViewPort {
            sign_column_width: 0,
            line_number: LineNumber::None,
            line_number_width: 0,
            prefix_column_width: 2,
            ..Default::default()
        };
        let bl = BufferLine::default();
        // prefix = 0, border = 0 (prefix is 0), custom_prefix = 2
        // offset = 0 + 0 + 2 = 2
        assert_eq!(vp.get_offset_width(&bl), 2);
    }

    #[test]
    fn content_width_reduced_by_prefix_column() {
        let vp = ViewPort {
            width: 80,
            sign_column_width: 0,
            line_number: LineNumber::None,
            line_number_width: 0,
            prefix_column_width: 0,
            ..Default::default()
        };
        let bl = BufferLine::default();
        let width_without_prefix = vp.get_content_width(&bl);

        let vp_with_prefix = ViewPort {
            width: 80,
            sign_column_width: 0,
            line_number: LineNumber::None,
            line_number_width: 0,
            prefix_column_width: 2,
            ..Default::default()
        };
        let width_with_prefix = vp_with_prefix.get_content_width(&bl);

        // With prefix_column_width=2, custom_prefix adds 2.
        // prefix is 0, border is 0, so offset = 2. Content reduced by 2.
        assert_eq!(
            width_without_prefix - width_with_prefix,
            2,
            "prefix column should reduce content width"
        );
    }

    #[test]
    fn custom_prefix_width_uses_column_width_when_set() {
        let vp = ViewPort {
            prefix_column_width: 2,
            ..Default::default()
        };
        let bl = BufferLine {
            prefix: Some("X".to_string()),
            ..Default::default()
        };
        assert_eq!(
            vp.get_custom_prefix_width(&bl),
            2,
            "should use prefix_column_width, not actual prefix length"
        );
    }

    #[test]
    fn custom_prefix_width_falls_back_to_prefix_len() {
        let vp = ViewPort {
            prefix_column_width: 0,
            ..Default::default()
        };
        let bl = BufferLine {
            prefix: Some("ABC".to_string()),
            ..Default::default()
        };
        assert_eq!(
            vp.get_custom_prefix_width(&bl),
            3,
            "should fall back to actual prefix char count"
        );
    }
}
