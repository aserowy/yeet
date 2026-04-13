use std::cmp::Reverse;

use crate::{
    model::{
        ansi::Ansi,
        viewport::{LineNumber, ViewPort},
        BufferLine, Cursor,
    },
    BufferTheme,
};

use super::style::{self, CUR_LINE_NR_BOLD};

pub fn get_border(vp: &ViewPort) -> Ansi {
    Ansi::new(&" ".repeat(vp.get_border_width()))
}

pub fn get_prefix_column(vp: &ViewPort, bl: &BufferLine, theme: &BufferTheme) -> Ansi {
    let width = vp.prefix_column_width;
    if width == 0 {
        return Ansi::new("");
    }

    let reset = style::ansi_reset_with_bg(theme.buffer_bg);

    match &bl.prefix {
        Some(prefix) => {
            let ansi = Ansi::new(prefix);
            let char_count = ansi.count_chars();
            if char_count < width {
                Ansi::new(&format!(
                    "{}{}{}{}",
                    reset,
                    " ".repeat(width - char_count),
                    prefix,
                    reset
                ))
            } else {
                Ansi::new(&format!("{}{}{}", reset, prefix, reset))
            }
        }
        None => Ansi::new(&format!("{}{}", reset, " ".repeat(width))),
    }
}

pub fn get_line_number(vp: &ViewPort, index: usize, cursor: &Cursor, theme: &BufferTheme) -> Ansi {
    if vp.line_number == LineNumber::None {
        return Ansi::new("");
    }

    let width = vp.get_line_number_width();
    let number = {
        let number_string = (index + 1).to_string();
        if let Some(index) = number_string.char_indices().nth_back(width - 1) {
            number_string[index.0..].to_string()
        } else {
            number_string
        }
    };

    let reset = style::ansi_reset_with_bg(theme.buffer_bg);

    if cursor.vertical_index == index {
        let cur_line_nr_fg = style::color_to_ansi_fg(theme.cur_line_nr);
        return Ansi::new(&format!(
            "{}{}{:<width$}{}",
            CUR_LINE_NR_BOLD, cur_line_nr_fg, number, reset
        ));
    }

    match vp.line_number {
        LineNumber::Absolute => {
            let line_nr_fg = style::color_to_ansi_fg(theme.line_nr);
            Ansi::new(&format!("{}{:>width$}{}", line_nr_fg, number, reset))
        }
        LineNumber::None => Ansi::new(""),
        LineNumber::Relative => {
            let relative = cursor.vertical_index.abs_diff(index);
            let line_nr_fg = style::color_to_ansi_fg(theme.line_nr);

            Ansi::new(&format!("{}{:>width$}{}", line_nr_fg, relative, reset))
        }
    }
}

pub fn get_signs(vp: &ViewPort, bl: &BufferLine, theme: &BufferTheme) -> Ansi {
    let max_sign_count = vp.sign_column_width;
    if max_sign_count == 0 {
        return Ansi::new("");
    }

    let reset = style::ansi_reset_with_bg(theme.buffer_bg);

    let mut filtered: Vec<_> = bl
        .signs
        .iter()
        .filter(|s| !vp.hidden_sign_ids.contains(&s.id))
        .collect();

    filtered.sort_unstable_by_key(|s| Reverse(s.priority));

    let signs_string = filtered
        .iter()
        .take(max_sign_count)
        .fold("".to_string(), |acc, s| {
            format!("{}{}{}{}", acc, s.style, s.content, reset)
        });

    let signs = Ansi::new(&signs_string);
    let char_count = signs.count_chars();
    if char_count < max_sign_count {
        Ansi::new(&format!(
            "{}{}{}",
            signs,
            " ".repeat(max_sign_count - char_count),
            reset
        ))
    } else {
        signs
    }
}

#[cfg(test)]
mod test {
    use ratatui::style::Color;

    use crate::{
        model::{
            viewport::{LineNumber, ViewPort},
            BufferLine, Cursor, CursorPosition, Sign,
        },
        BufferTheme,
    };

    use super::{get_line_number, get_prefix_column, get_signs};

    fn test_theme() -> BufferTheme {
        BufferTheme {
            buffer_bg: Color::Reset,
            buffer_fg: Color::White,
            cursor_line_bg: Color::Rgb(128, 128, 128),
            search_bg: Color::Red,
            line_nr: Color::Rgb(128, 128, 128),
            cur_line_nr: Color::White,
            border_fg: Color::Black,
            border_bg: Color::Reset,
        }
    }

    #[test]
    fn prefix_column_width_zero_returns_empty() {
        let vp = ViewPort {
            prefix_column_width: 0,
            ..Default::default()
        };
        let bl = BufferLine::default();
        let result = get_prefix_column(&vp, &bl, &test_theme());
        assert_eq!(result.count_chars(), 0, "width 0 should produce no output");
    }

    #[test]
    fn prefix_column_no_prefix_renders_spaces() {
        let vp = ViewPort {
            prefix_column_width: 2,
            ..Default::default()
        };
        let bl = BufferLine::default();
        let result = get_prefix_column(&vp, &bl, &test_theme());
        assert_eq!(
            result.count_chars(),
            2,
            "width 2 with no prefix should render two spaces"
        );
        assert_eq!(
            result.to_stripped_string(),
            "  ",
            "fallback should be spaces"
        );
    }

    #[test]
    fn prefix_column_renders_prefix_right_aligned() {
        let vp = ViewPort {
            prefix_column_width: 2,
            ..Default::default()
        };
        let bl = BufferLine {
            prefix: Some("\u{f0f6}".to_string()),
            ..Default::default()
        };
        let result = get_prefix_column(&vp, &bl, &test_theme());
        let stripped = result.to_stripped_string();
        assert!(
            stripped.starts_with(' '),
            "single-char prefix in width-2 column should be right-aligned with leading space"
        );
        assert!(
            stripped.contains('\u{f0f6}'),
            "prefix glyph should appear in rendered output"
        );
    }

    #[test]
    fn prefix_column_applies_color_from_prefix_string() {
        let vp = ViewPort {
            prefix_column_width: 2,
            ..Default::default()
        };
        let bl = BufferLine {
            prefix: Some("\x1b[38;2;255;0;0m\u{f0f6}\x1b[0m".to_string()),
            ..Default::default()
        };
        let result = get_prefix_column(&vp, &bl, &test_theme());
        let raw = format!("{}", result);
        assert!(
            raw.contains("\x1b[38;2;255;0;0m"),
            "ANSI color in prefix string should be present in rendered output"
        );
    }

    #[test]
    fn prefix_column_no_color_still_renders_prefix() {
        let vp = ViewPort {
            prefix_column_width: 2,
            ..Default::default()
        };
        let bl = BufferLine {
            prefix: Some("\u{f0f6}".to_string()),
            ..Default::default()
        };
        let result = get_prefix_column(&vp, &bl, &test_theme());
        assert!(
            result.to_stripped_string().contains('\u{f0f6}'),
            "prefix glyph should render even without ANSI color"
        );
    }

    #[test]
    fn prefix_column_zero_width_with_prefix_returns_empty() {
        let vp = ViewPort {
            prefix_column_width: 0,
            ..Default::default()
        };
        let bl = BufferLine {
            prefix: Some("X".to_string()),
            ..Default::default()
        };
        let result = get_prefix_column(&vp, &bl, &test_theme());
        assert_eq!(
            result.count_chars(),
            0,
            "zero width with prefix should suppress rendering and return empty"
        );
    }

    #[test]
    fn absolute_line_number_cursor_and_non_cursor_same_width() {
        let vp = ViewPort {
            line_number: LineNumber::Absolute,
            line_number_width: 3,
            ..Default::default()
        };
        let cursor = Cursor {
            vertical_index: 0,
            horizontal_index: CursorPosition::Absolute {
                current: 0,
                expanded: 0,
            },
        };
        let theme = test_theme();

        let cursor_line = get_line_number(&vp, 0, &cursor, &theme);
        let non_cursor_line = get_line_number(&vp, 1, &cursor, &theme);

        assert_eq!(
            cursor_line.count_chars(),
            non_cursor_line.count_chars(),
            "cursor line number ({}) and non-cursor line number ({}) should have the same visible width",
            cursor_line.count_chars(),
            non_cursor_line.count_chars(),
        );
        assert_eq!(
            cursor_line.count_chars(),
            vp.get_line_number_width(),
            "line number visible width should equal configured line_number_width",
        );
    }

    #[test]
    fn relative_line_number_cursor_and_non_cursor_same_width() {
        let vp = ViewPort {
            line_number: LineNumber::Relative,
            line_number_width: 3,
            ..Default::default()
        };
        let cursor = Cursor {
            vertical_index: 0,
            horizontal_index: CursorPosition::Absolute {
                current: 0,
                expanded: 0,
            },
        };
        let theme = test_theme();

        let cursor_line = get_line_number(&vp, 0, &cursor, &theme);
        let non_cursor_line = get_line_number(&vp, 1, &cursor, &theme);

        assert_eq!(
            cursor_line.count_chars(),
            non_cursor_line.count_chars(),
            "cursor line number ({}) and non-cursor line number ({}) should have the same visible width in relative mode",
            cursor_line.count_chars(),
            non_cursor_line.count_chars(),
        );
    }

    #[test]
    fn signs_output_ends_with_ansi_reset_when_signs_present() {
        let vp = ViewPort {
            sign_column_width: 2,
            ..Default::default()
        };
        let bl = BufferLine {
            signs: vec![Sign {
                id: "test",
                content: '▶',
                priority: 1,
                style: "\x1b[31m".to_string(),
            }],
            ..Default::default()
        };
        let theme = test_theme();
        let result = get_signs(&vp, &bl, &theme);
        let raw = format!("{}", result);
        assert!(
            raw.ends_with("\x1b[0m\x1b[49m"),
            "get_signs() with signs present should end with ANSI reset, got: {:?}",
            raw,
        );
    }

    #[test]
    fn signs_output_ends_with_ansi_reset_when_no_signs() {
        let vp = ViewPort {
            sign_column_width: 2,
            ..Default::default()
        };
        let bl = BufferLine::default();
        let theme = test_theme();
        let result = get_signs(&vp, &bl, &theme);
        let raw = format!("{}", result);
        assert!(
            raw.ends_with("\x1b[0m\x1b[49m"),
            "get_signs() with no signs should end with ANSI reset, got: {:?}",
            raw,
        );
    }

    #[test]
    fn absolute_non_cursor_line_number_ends_with_ansi_reset() {
        let vp = ViewPort {
            line_number: LineNumber::Absolute,
            line_number_width: 3,
            ..Default::default()
        };
        let cursor = Cursor {
            vertical_index: 0,
            horizontal_index: CursorPosition::Absolute {
                current: 0,
                expanded: 0,
            },
        };
        let theme = test_theme();
        let result = get_line_number(&vp, 1, &cursor, &theme);
        let raw = format!("{}", result);
        assert!(
            raw.ends_with("\x1b[0m\x1b[49m"),
            "non-cursor absolute line number should end with ANSI reset, got: {:?}",
            raw,
        );
    }

    #[test]
    fn absolute_non_cursor_line_number_contains_ansi_fg() {
        let vp = ViewPort {
            line_number: LineNumber::Absolute,
            line_number_width: 3,
            ..Default::default()
        };
        let cursor = Cursor {
            vertical_index: 0,
            horizontal_index: CursorPosition::Absolute {
                current: 0,
                expanded: 0,
            },
        };
        let theme = test_theme();
        let result = get_line_number(&vp, 1, &cursor, &theme);
        let raw = format!("{}", result);
        assert!(
            raw.contains("\x1b[38;2;128;128;128m"),
            "non-cursor absolute line number should contain fg color ANSI code, got: {:?}",
            raw,
        );
    }
}
