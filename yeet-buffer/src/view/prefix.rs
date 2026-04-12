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
        if let Some(prefix) = &bl.prefix {
            return Ansi::new(prefix);
        }
        return Ansi::new("");
    }

    let reset = style::ansi_reset_with_bg(theme.buffer_bg);

    match &bl.prefix {
        Some(prefix) => {
            let ansi = Ansi::new(prefix);
            let char_count = ansi.count_chars();
            if char_count < width {
                Ansi::new(&format!(
                    "{}{}{}",
                    " ".repeat(width - char_count),
                    prefix,
                    reset
                ))
            } else {
                Ansi::new(&format!("{}{}", prefix, reset))
            }
        }
        None => Ansi::new(&" ".repeat(width)),
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
        LineNumber::Absolute => Ansi::new(&format!("{:>width$} ", number)),
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
            "{}{}",
            signs,
            " ".repeat(max_sign_count - char_count)
        ))
    } else {
        signs
    }
}

#[cfg(test)]
mod test {
    use ratatui::style::Color;

    use crate::{
        model::{viewport::ViewPort, BufferLine},
        BufferTheme,
    };

    use super::get_prefix_column;

    fn test_theme() -> BufferTheme {
        BufferTheme {
            buffer_bg: Color::Reset,
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
    fn prefix_column_zero_width_with_prefix_returns_prefix() {
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
            result.to_stripped_string(),
            "X",
            "zero width with prefix should return prefix as-is"
        );
    }
}
