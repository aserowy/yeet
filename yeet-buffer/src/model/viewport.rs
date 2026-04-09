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
    /// Width of the icon column prefix segment. Defaults to `0`; set to `1` by
    /// the directory-icons plugin via `on_window_create` hook.
    pub icon_column_width: usize,
    pub line_number: LineNumber,
    pub line_number_width: usize,
    pub show_border: bool,
    pub sign_column_width: usize,
    pub vertical_index: usize,
    pub width: u16,
    pub wrap: bool,
    pub x: u16,
    pub y: u16,
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
        let custom_prefix_width = if let Some(prefix) = &line.prefix {
            prefix.chars().count()
        } else {
            0
        };

        self.get_prefix_width() + self.get_border_width() + custom_prefix_width
    }

    fn get_prefix_width(&self) -> usize {
        self.sign_column_width + self.get_line_number_width() + self.icon_column_width
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
    fn icon_column_width_defaults_to_zero() {
        let vp = ViewPort::default();
        assert_eq!(vp.icon_column_width, 0);
    }

    #[test]
    fn prefix_width_excludes_icon_column_when_zero() {
        let vp = ViewPort {
            sign_column_width: 2,
            line_number: LineNumber::Absolute,
            line_number_width: 3,
            icon_column_width: 0,
            ..Default::default()
        };
        // prefix = sign(2) + line_number(3) + icon(0) = 5
        assert_eq!(vp.get_prefix_width(), 5);
    }

    #[test]
    fn prefix_width_includes_icon_column_when_set() {
        let vp = ViewPort {
            sign_column_width: 2,
            line_number: LineNumber::Absolute,
            line_number_width: 3,
            icon_column_width: 1,
            ..Default::default()
        };
        // prefix = sign(2) + line_number(3) + icon(1) = 6
        assert_eq!(vp.get_prefix_width(), 6);
    }

    #[test]
    fn offset_width_includes_icon_column() {
        let vp = ViewPort {
            sign_column_width: 0,
            line_number: LineNumber::None,
            line_number_width: 0,
            icon_column_width: 1,
            ..Default::default()
        };
        let bl = BufferLine::default();
        // prefix = 0 + 0 + 1 = 1, border = 1 (prefix > 0), custom = 0
        // offset = 1 + 1 + 0 = 2
        assert_eq!(vp.get_offset_width(&bl), 2);
    }

    #[test]
    fn content_width_reduced_by_icon_column() {
        let vp = ViewPort {
            width: 80,
            sign_column_width: 0,
            line_number: LineNumber::None,
            line_number_width: 0,
            icon_column_width: 0,
            ..Default::default()
        };
        let bl = BufferLine::default();
        let width_without_icon = vp.get_content_width(&bl);

        let vp_with_icon = ViewPort {
            width: 80,
            sign_column_width: 0,
            line_number: LineNumber::None,
            line_number_width: 0,
            icon_column_width: 1,
            ..Default::default()
        };
        let width_with_icon = vp_with_icon.get_content_width(&bl);

        // With icon_column_width=1, prefix becomes non-zero so border(1) appears.
        // get_content_width subtracts offset (prefix + border) and border again,
        // so content is reduced by icon(1) + border(1) + border(1) = 3.
        assert_eq!(
            width_without_icon - width_with_icon,
            3,
            "icon column + border overhead should reduce content width by 3"
        );
    }
}
