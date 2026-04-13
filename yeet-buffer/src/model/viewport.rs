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
    pub precontent_border_width: Option<usize>,
    pub show_border: bool,
    pub sign_column_width: usize,
    pub vertical_index: usize,
    pub width: u16,
    pub wrap: bool,
    pub x: u16,
    pub y: u16,
}

impl ViewPort {
    pub fn get_precontent_border_width(&self) -> usize {
        if let Some(override_width) = self.precontent_border_width {
            return override_width;
        }
        if self.get_precontent_width() > 0 {
            1
        } else {
            0
        }
    }

    pub fn get_content_width(&self, line: &BufferLine) -> usize {
        usize::from(self.width)
            .saturating_sub(self.get_offset_width(line))
            .saturating_sub(self.get_precontent_border_width())
    }

    pub fn get_line_number_width(&self) -> usize {
        match self.line_number {
            LineNumber::Absolute => self.line_number_width,
            LineNumber::None => 0,
            LineNumber::Relative => self.line_number_width,
        }
    }

    pub fn get_offset_width(&self, _line: &BufferLine) -> usize {
        self.get_precontent_width() + self.get_precontent_border_width()
    }

    pub fn get_precontent_width(&self) -> usize {
        self.sign_column_width + self.get_line_number_width() + self.prefix_column_width
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
    fn precontent_width_includes_all_prefix_columns() {
        let vp = ViewPort {
            sign_column_width: 2,
            line_number: LineNumber::Absolute,
            line_number_width: 3,
            prefix_column_width: 0,
            ..Default::default()
        };
        assert_eq!(vp.get_precontent_width(), 5);
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
        // precontent = sign(2) + line_number(3) + prefix_column(2) = 7, border = 1
        // offset = 7 + 1 = 8
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
        // precontent = 0 + 0 + 2 = 2, border = 1 (precontent > 0)
        // offset = 2 + 1 = 3
        assert_eq!(vp.get_offset_width(&bl), 3);
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

        // precontent = 2, border = 1, offset = 3.
        // get_content_width also subtracts border (for ratatui Borders::RIGHT) = 1.
        // total reduction = 3 + 1 = 4.
        assert_eq!(
            width_without_prefix - width_with_prefix,
            4,
            "prefix column should reduce content width by prefix_column + 2*border"
        );
    }

    #[test]
    fn precontent_width_includes_prefix_column() {
        let vp = ViewPort {
            prefix_column_width: 2,
            ..Default::default()
        };
        assert_eq!(
            vp.get_precontent_width(),
            2,
            "precontent_width should include prefix_column_width"
        );
    }

    #[test]
    fn precontent_width_zero_when_no_columns() {
        let vp = ViewPort {
            prefix_column_width: 0,
            ..Default::default()
        };
        assert_eq!(
            vp.get_precontent_width(),
            0,
            "should return 0 when all widths are 0"
        );
    }

    #[test]
    fn precontent_border_width_one_when_only_prefix_column_active() {
        let vp = ViewPort {
            sign_column_width: 0,
            line_number: LineNumber::None,
            line_number_width: 0,
            prefix_column_width: 2,
            ..Default::default()
        };
        assert_eq!(
            vp.get_precontent_border_width(),
            1,
            "border should be 1 when only prefix_column_width > 0"
        );
    }

    #[test]
    fn precontent_border_width_zero_when_all_columns_zero() {
        let vp = ViewPort {
            sign_column_width: 0,
            line_number: LineNumber::None,
            line_number_width: 0,
            prefix_column_width: 0,
            ..Default::default()
        };
        assert_eq!(
            vp.get_precontent_border_width(),
            0,
            "border should be 0 when all pre-content columns are zero"
        );
    }

    #[test]
    fn precontent_border_width_override_zero_suppresses_border() {
        let vp = ViewPort {
            sign_column_width: 0,
            line_number: LineNumber::None,
            prefix_column_width: 2,
            precontent_border_width: Some(0),
            ..Default::default()
        };
        assert_eq!(
            vp.get_precontent_border_width(),
            0,
            "override Some(0) should suppress border even with precontent > 0"
        );
    }

    #[test]
    fn precontent_border_width_override_enforces_value() {
        let vp = ViewPort {
            sign_column_width: 0,
            line_number: LineNumber::None,
            prefix_column_width: 0,
            precontent_border_width: Some(2),
            ..Default::default()
        };
        assert_eq!(
            vp.get_precontent_border_width(),
            2,
            "override Some(2) should return 2 regardless of precontent width"
        );
    }

    #[test]
    fn precontent_border_width_none_uses_computed() {
        let vp = ViewPort {
            sign_column_width: 2,
            precontent_border_width: None,
            ..Default::default()
        };
        assert_eq!(
            vp.get_precontent_border_width(),
            1,
            "None should fall back to computed value (1 when precontent > 0)"
        );
    }
}
