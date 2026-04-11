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

pub fn get_custom_prefix(line: &BufferLine) -> Ansi {
    if let Some(prefix) = &line.prefix {
        Ansi::new(prefix)
    } else {
        Ansi::new("")
    }
}

/// Renders the icon column prefix segment for a bufferline.
///
/// If the viewport `icon_column_width` is `0`, returns an empty string.
/// Otherwise, renders the bufferline's `icon` glyph (if set by a plugin
/// mutation hook) as-is, or empty space as fallback.
pub fn get_icon_column(vp: &ViewPort, bl: &BufferLine, theme: &BufferTheme) -> Ansi {
    let width = vp.icon_column_width;
    if width == 0 {
        return Ansi::new("");
    }

    let reset = style::ansi_reset_with_bg(theme.buffer_bg);

    match &bl.icon {
        Some(icon) => Ansi::new(&format!("{}{}", icon, reset)),
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

    use super::get_icon_column;

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
    fn icon_column_width_zero_returns_empty() {
        let vp = ViewPort {
            icon_column_width: 0,
            ..Default::default()
        };
        let bl = BufferLine::default();
        let result = get_icon_column(&vp, &bl, &test_theme());
        assert_eq!(result.count_chars(), 0, "width 0 should produce no output");
    }

    #[test]
    fn icon_column_no_icon_renders_space() {
        let vp = ViewPort {
            icon_column_width: 1,
            ..Default::default()
        };
        let bl = BufferLine::default();
        let result = get_icon_column(&vp, &bl, &test_theme());
        assert_eq!(
            result.count_chars(),
            1,
            "width 1 with no icon should render one space"
        );
        assert_eq!(
            result.to_stripped_string(),
            " ",
            "fallback icon should be a space"
        );
    }

    #[test]
    fn icon_column_renders_icon_glyph() {
        let vp = ViewPort {
            icon_column_width: 1,
            ..Default::default()
        };
        let bl = BufferLine {
            icon: Some("\u{f0f6}".to_string()),
            ..Default::default()
        };
        let result = get_icon_column(&vp, &bl, &test_theme());
        assert!(
            result.to_stripped_string().contains('\u{f0f6}'),
            "icon glyph should appear in rendered output"
        );
    }

    #[test]
    fn icon_column_applies_color_from_icon_string() {
        let vp = ViewPort {
            icon_column_width: 1,
            ..Default::default()
        };
        let bl = BufferLine {
            icon: Some("\x1b[38;2;255;0;0m\u{f0f6}\x1b[0m".to_string()),
            ..Default::default()
        };
        let result = get_icon_column(&vp, &bl, &test_theme());
        let raw = format!("{}", result);
        assert!(
            raw.contains("\x1b[38;2;255;0;0m"),
            "ANSI color in icon string should be present in rendered output"
        );
    }

    #[test]
    fn icon_column_no_color_still_renders_icon() {
        let vp = ViewPort {
            icon_column_width: 1,
            ..Default::default()
        };
        let bl = BufferLine {
            icon: Some("\u{f0f6}".to_string()),
            ..Default::default()
        };
        let result = get_icon_column(&vp, &bl, &test_theme());
        assert!(
            result.to_stripped_string().contains('\u{f0f6}'),
            "icon glyph should render even without ANSI color"
        );
    }
}
