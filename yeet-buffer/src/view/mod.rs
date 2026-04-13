use ansi_to_tui::IntoText;
use ratatui::{
    prelude::Rect,
    style::Style,
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{
    model::{ansi::Ansi, viewport::ViewPort, BufferLine, Cursor, Mode, TextBuffer},
    BufferTheme,
};

mod line;
mod prefix;
pub(crate) mod style;
pub(crate) mod wrap;

pub fn view(
    viewport: &ViewPort,
    mode: &Mode,
    buffer: &TextBuffer,
    theme: &BufferTheme,
    frame: &mut Frame,
) {
    let rendered = get_rendered_lines(viewport, buffer);
    let styled = get_styled_lines(viewport, mode, &viewport.cursor, rendered, theme);

    let rect = Rect {
        x: viewport.x,
        y: viewport.y,
        width: viewport.width,
        height: viewport.height,
    };

    let rect = if viewport.show_border {
        let block = Block::default()
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(theme.border_fg).bg(theme.border_bg));

        let inner = block.inner(rect);

        frame.render_widget(block, rect);

        inner
    } else {
        rect
    };

    frame.render_widget(
        Paragraph::new(styled).style(Style::default().fg(theme.buffer_fg).bg(theme.buffer_bg)),
        rect,
    );
}

fn get_rendered_lines(viewport: &ViewPort, buffer: &TextBuffer) -> Vec<BufferLine> {
    if !viewport.wrap {
        return buffer
            .lines
            .iter()
            .skip(viewport.vertical_index)
            .take(usize::from(viewport.height))
            .map(|line| line.to_owned())
            .collect();
    }

    let height = usize::from(viewport.height);
    let mut result = Vec::new();
    let mut visual_rows = 0;

    for line in buffer.lines.iter().skip(viewport.vertical_index) {
        let line_visual_height =
            wrap::visual_line_count(&line.content, viewport.get_content_width(line));
        if visual_rows + line_visual_height > height && !result.is_empty() {
            break;
        }
        result.push(line.to_owned());
        visual_rows += line_visual_height;
        if visual_rows >= height {
            break;
        }
    }

    result
}

fn get_styled_lines<'a>(
    vp: &ViewPort,
    mode: &Mode,
    cursor: &Cursor,
    lines: Vec<BufferLine>,
    theme: &BufferTheme,
) -> Vec<Line<'a>> {
    let lines = if lines.is_empty() {
        vec![BufferLine::default()]
    } else {
        lines
    };

    if !vp.wrap {
        return get_styled_lines_nowrap(vp, mode, cursor, lines, theme);
    }

    let mut result = Vec::new();
    let mut visual_row = 0;

    for (i, bl) in lines.into_iter().enumerate() {
        let corrected_index = i + vp.vertical_index;
        let content_width = vp.get_content_width(&bl);
        let segments = wrap::wrap_line(&bl.content, content_width);

        for segment in &segments {
            if visual_row >= usize::from(vp.height) {
                break;
            }

            let mut prefix = if segment.is_first {
                Ansi::new("")
                    .join(&prefix::get_signs(vp, &bl, theme))
                    .join(&prefix::get_line_number(vp, corrected_index, cursor, theme))
                    .join(&prefix::get_prefix_column(vp, &bl, theme))
                    .join(&prefix::get_border(vp))
            } else {
                let prefix_width = vp.get_offset_width(&bl) + vp.get_precontent_border_width();
                Ansi::new(&" ".repeat(prefix_width))
            };

            let mut segment_bl = BufferLine {
                content: segment.content.clone(),
                search_char_position: None,
                signs: Vec::new(),
                prefix: None,
            };

            let content = prefix.join(&line::add_line_styles_wrap(
                vp,
                mode,
                cursor,
                &i,
                &mut segment_bl,
                theme,
                content_width,
                segment.char_offset,
            ));

            if let Ok(text) = content.to_string().into_text() {
                result.push(text.lines);
            }

            visual_row += 1;
        }
    }

    result.into_iter().flatten().collect()
}

fn get_styled_lines_nowrap<'a>(
    vp: &ViewPort,
    mode: &Mode,
    cursor: &Cursor,
    lines: Vec<BufferLine>,
    theme: &BufferTheme,
) -> Vec<Line<'a>> {
    let mut result = Vec::new();
    for (i, mut bl) in lines.into_iter().enumerate() {
        let corrected_index = i + vp.vertical_index;

        let content = Ansi::new("")
            .join(&prefix::get_signs(vp, &bl, theme))
            .join(&prefix::get_line_number(vp, corrected_index, cursor, theme))
            .join(&prefix::get_prefix_column(vp, &bl, theme))
            .join(&prefix::get_border(vp))
            .join(&line::add_line_styles(vp, mode, cursor, &i, &mut bl, theme));

        if let Ok(text) = content.to_string().into_text() {
            result.push(text.lines);
        }
    }

    result.into_iter().flatten().collect()
}

#[cfg(test)]
mod test {
    use crate::{
        model::{
            viewport::{LineNumber, ViewPort},
            BufferLine, Cursor, CursorPosition, Mode,
        },
        BufferTheme,
    };

    use super::get_styled_lines;

    fn test_theme() -> BufferTheme {
        use ratatui::style::Color;
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

    fn tasks_viewport(width: u16, height: u16) -> ViewPort {
        ViewPort {
            width,
            height,
            sign_column_width: 0,
            line_number: LineNumber::None,
            line_number_width: 0,
            show_border: false,
            hide_cursor: false,
            hide_cursor_line: false,
            cursor: Cursor {
                vertical_index: 0,
                horizontal_index: CursorPosition::Absolute {
                    current: 0,
                    expanded: 0,
                },
            },
            ..Default::default()
        }
    }

    fn directory_current_viewport(width: u16, height: u16) -> ViewPort {
        ViewPort {
            width,
            height,
            sign_column_width: 2,
            line_number: LineNumber::Relative,
            line_number_width: 3,
            show_border: true,
            hide_cursor: false,
            hide_cursor_line: false,
            cursor: Cursor {
                vertical_index: 0,
                horizontal_index: CursorPosition::Absolute {
                    current: 0,
                    expanded: 0,
                },
            },
            ..Default::default()
        }
    }

    #[test]
    fn cursor_line_width_equals_viewport_width_for_tasks() {
        let vp = tasks_viewport(80, 10);
        let lines = vec![BufferLine::from("1    rg foo")];

        let styled = get_styled_lines(&vp, &Mode::Navigation, &vp.cursor, lines, &test_theme());

        assert!(!styled.is_empty(), "should produce at least one line");
        let cursor_line = &styled[0];
        assert_eq!(
            cursor_line.width(),
            usize::from(vp.width),
            "cursor line width should equal viewport width (no prefix, no border)"
        );
    }

    #[test]
    fn cursor_line_width_equals_viewport_width_for_tasks_empty_line() {
        let vp = tasks_viewport(80, 10);
        let lines = vec![BufferLine::default()];

        let styled = get_styled_lines(&vp, &Mode::Navigation, &vp.cursor, lines, &test_theme());

        assert!(!styled.is_empty());
        let cursor_line = &styled[0];
        assert_eq!(
            cursor_line.width(),
            usize::from(vp.width),
            "cursor line width on empty content should equal viewport width"
        );
    }

    #[test]
    fn cursor_line_width_equals_viewport_width_for_directory_current() {
        let vp = directory_current_viewport(40, 10);
        let lines = vec![BufferLine::from("documents")];

        let styled = get_styled_lines(&vp, &Mode::Navigation, &vp.cursor, lines, &test_theme());

        assert!(!styled.is_empty());
        let cursor_line = &styled[0];

        // For a directory viewport with show_border=true, the Paragraph rect
        // is reduced by Block::inner() (Borders::RIGHT takes 1 col).
        // The styled line should fill that inner rect: viewport.width - 1.
        let expected_width = usize::from(vp.width) - 1;
        assert_eq!(
            cursor_line.width(),
            expected_width,
            "cursor line width should equal viewport width minus border (sign_col=2, line_num=3, border=1, content=width-7, total=width-1)"
        );
    }

    #[test]
    fn non_cursor_lines_do_not_exceed_viewport_width_for_tasks() {
        let mut vp = tasks_viewport(80, 10);
        vp.cursor.vertical_index = 1; // cursor on second line

        let lines = vec![
            BufferLine::from("1    rg foo"),
            BufferLine::from("2    fd bar"),
        ];

        let styled = get_styled_lines(&vp, &Mode::Navigation, &vp.cursor, lines, &test_theme());

        assert!(styled.len() >= 2);
        // First line is NOT the cursor line (cursor is on index 1)
        let non_cursor_line = &styled[0];
        assert!(
            non_cursor_line.width() <= usize::from(vp.width),
            "non-cursor line width ({}) should not exceed viewport width ({})",
            non_cursor_line.width(),
            vp.width,
        );

        // Second line IS the cursor line
        let cursor_line = &styled[1];
        assert_eq!(
            cursor_line.width(),
            usize::from(vp.width),
            "cursor line width should equal viewport width"
        );
    }

    #[test]
    fn cursor_line_width_with_cancelled_task_ansi() {
        let vp = tasks_viewport(80, 10);
        // Cancelled task line with strikethrough + gray ANSI styling
        let lines = vec![BufferLine::from("\x1b[9;90m1    rg foo\x1b[0m")];

        let styled = get_styled_lines(&vp, &Mode::Navigation, &vp.cursor, lines, &test_theme());

        assert!(!styled.is_empty());
        let cursor_line = &styled[0];
        assert_eq!(
            cursor_line.width(),
            usize::from(vp.width),
            "cursor line width with ANSI-styled content should equal viewport width"
        );
    }

    #[test]
    fn trailing_newline_in_ansi_produces_extra_line() {
        let vp = tasks_viewport(80, 10);
        let lines = vec![BufferLine::from("\x1b[38;2;255;100;50m# Commands\n\x1b[0m")];

        let styled = get_styled_lines(&vp, &Mode::Navigation, &vp.cursor, lines, &test_theme());

        assert_eq!(
            styled.len(),
            2,
            "a BufferLine with embedded newline produces two rendered lines via ansi_to_tui"
        );
        assert!(
            styled[0].width() < usize::from(vp.width),
            "first rendered line is shorter than viewport because padding was split across lines"
        );
    }

    #[test]
    fn no_trailing_newline_cursor_line_fills_viewport() {
        let vp = tasks_viewport(80, 10);
        let lines = vec![BufferLine::from("\x1b[38;2;255;100;50m# Commands\x1b[0m")];

        let styled = get_styled_lines(&vp, &Mode::Navigation, &vp.cursor, lines, &test_theme());

        assert_eq!(styled.len(), 1);
        assert_eq!(
            styled[0].width(),
            usize::from(vp.width),
            "cursor line without trailing newline should fill viewport width"
        );
    }

    #[test]
    fn cursor_line_bg_preserved_through_ansi_reset() {
        use ratatui::style::Color;

        let vp = tasks_viewport(80, 10);
        let lines = vec![BufferLine::from("\x1b[1m1    rg foo\x1b[0m")];

        let styled = get_styled_lines(&vp, &Mode::Navigation, &vp.cursor, lines, &test_theme());

        assert!(!styled.is_empty());
        let cursor_line = &styled[0];
        let expected_bg = Color::Rgb(128, 128, 128);
        for span in &cursor_line.spans {
            assert_eq!(
                span.style.bg.unwrap_or(expected_bg),
                expected_bg,
                "every span on cursor line should have cursor_line_bg, got {:?} in span {:?}",
                span.style.bg,
                span.content,
            );
        }
    }

    #[test]
    fn unfocused_buffer_bg_preserved_through_ansi_reset() {
        use ratatui::style::Color;

        let mut vp = tasks_viewport(80, 10);
        vp.hide_cursor_line = true;
        vp.hide_cursor = true;
        let lines = vec![BufferLine::from("\x1b[1m1    rg foo\x1b[0m")];

        let styled = get_styled_lines(&vp, &Mode::Navigation, &vp.cursor, lines, &test_theme());

        assert!(!styled.is_empty());
        let cursor_line = &styled[0];
        for span in &cursor_line.spans {
            assert!(
                span.style.bg.is_none() || span.style.bg == Some(Color::Reset),
                "unfocused line spans should not override buffer_bg, got {:?} in span {:?}",
                span.style.bg,
                span.content,
            );
        }
    }

    #[test]
    fn cursor_line_width_in_normal_mode() {
        let vp = tasks_viewport(80, 10);
        let lines = vec![BufferLine::from("1    rg foo")];

        let styled = get_styled_lines(&vp, &Mode::Normal, &vp.cursor, lines, &test_theme());

        assert!(!styled.is_empty());
        let cursor_line = &styled[0];
        assert_eq!(
            cursor_line.width(),
            usize::from(vp.width),
            "cursor line width in Normal mode should equal viewport width"
        );
    }

    fn wrap_viewport(width: u16, height: u16) -> ViewPort {
        ViewPort {
            width,
            height,
            sign_column_width: 0,
            line_number: LineNumber::None,
            line_number_width: 0,
            show_border: false,
            hide_cursor: false,
            hide_cursor_line: false,
            wrap: true,
            cursor: Cursor {
                vertical_index: 0,
                horizontal_index: CursorPosition::Absolute {
                    current: 0,
                    expanded: 0,
                },
            },
            ..Default::default()
        }
    }

    #[test]
    fn wrap_produces_multiple_lines() {
        let vp = wrap_viewport(10, 10);
        let lines = vec![BufferLine::from("hello world foo bar")];

        let styled = get_styled_lines(&vp, &Mode::Navigation, &vp.cursor, lines, &test_theme());

        assert!(
            styled.len() > 1,
            "a wrapped long line should produce multiple visual lines, got {}",
            styled.len()
        );
    }

    #[test]
    fn wrap_short_line_single_output() {
        let vp = wrap_viewport(80, 10);
        let lines = vec![BufferLine::from("hello")];

        let styled = get_styled_lines(&vp, &Mode::Navigation, &vp.cursor, lines, &test_theme());

        assert_eq!(styled.len(), 1);
    }

    #[test]
    fn wrap_cursor_line_bg_spans_all_segments() {
        use ratatui::style::Color;

        let vp = wrap_viewport(10, 10);
        let lines = vec![BufferLine::from("hello world foo bar")];

        let styled = get_styled_lines(&vp, &Mode::Navigation, &vp.cursor, lines, &test_theme());

        assert!(styled.len() > 1);
        let expected_bg = Color::Rgb(128, 128, 128);
        for (row, line) in styled.iter().enumerate() {
            for span in &line.spans {
                assert_eq!(
                    span.style.bg.unwrap_or(expected_bg),
                    expected_bg,
                    "row {} span {:?} should have cursor_line_bg",
                    row,
                    span.content,
                );
            }
        }
    }

    #[test]
    fn wrap_nowrap_viewport_unchanged() {
        let mut vp = tasks_viewport(80, 10);
        vp.wrap = false;
        let lines = vec![BufferLine::from("hello world foo bar")];

        let styled = get_styled_lines(&vp, &Mode::Navigation, &vp.cursor, lines, &test_theme());

        assert_eq!(styled.len(), 1);
    }

    #[test]
    fn wrap_each_segment_fills_viewport_width() {
        let vp = wrap_viewport(10, 10);
        let lines = vec![BufferLine::from("hello world foo bar")];

        let styled = get_styled_lines(&vp, &Mode::Navigation, &vp.cursor, lines, &test_theme());

        for (row, line) in styled.iter().enumerate() {
            assert_eq!(
                line.width(),
                usize::from(vp.width),
                "row {} should fill viewport width",
                row,
            );
        }
    }

    #[test]
    fn prefix_column_included_in_directory_viewport_line_width() {
        let mut vp = directory_current_viewport(40, 10);
        vp.prefix_column_width = 2;
        let lines = vec![BufferLine {
            prefix: Some("\u{f0f6}".to_string()),
            ..BufferLine::from("documents")
        }];

        let styled = get_styled_lines(&vp, &Mode::Navigation, &vp.cursor, lines, &test_theme());

        assert!(!styled.is_empty());
        let cursor_line = &styled[0];
        // With show_border=true, the inner rect is viewport.width - 1
        let expected_width = usize::from(vp.width) - 1;
        assert_eq!(
            cursor_line.width(),
            expected_width,
            "cursor line width should equal viewport width minus border, including prefix column"
        );
    }

    #[test]
    fn prefix_column_zero_width_does_not_affect_line_width() {
        let mut vp = directory_current_viewport(40, 10);
        vp.prefix_column_width = 0;
        let lines = vec![BufferLine::from("documents")];

        let styled = get_styled_lines(&vp, &Mode::Navigation, &vp.cursor, lines, &test_theme());

        assert!(!styled.is_empty());
        let cursor_line = &styled[0];
        let expected_width = usize::from(vp.width) - 1;
        assert_eq!(
            cursor_line.width(),
            expected_width,
            "prefix_column_width=0 should not change line width"
        );
    }

    #[test]
    fn plugin_ansi_in_content_applied_to_text() {
        use ratatui::style::Color;

        let vp = tasks_viewport(80, 10);
        let lines = vec![BufferLine {
            content: crate::model::ansi::Ansi::new("\x1b[38;2;255;100;0mmyfile.rs"),
            ..Default::default()
        }];

        let styled = get_styled_lines(&vp, &Mode::Navigation, &vp.cursor, lines, &test_theme());

        assert!(!styled.is_empty());
        // The plugin-prepended ANSI code should be visible in the rendered spans
        let has_fg_color = styled[0]
            .spans
            .iter()
            .any(|span| span.style.fg.is_some() && span.style.fg != Some(Color::Reset));
        assert!(
            has_fg_color,
            "plugin ANSI in content should color the text (some span should have a non-default fg)"
        );
    }

    #[test]
    fn plain_content_uses_default_styling() {
        let vp = tasks_viewport(80, 10);
        let lines = vec![BufferLine::from("myfile.rs")];

        let styled = get_styled_lines(&vp, &Mode::Navigation, &vp.cursor, lines, &test_theme());

        assert!(!styled.is_empty());
        // Without any ANSI in content, the text should not have a custom fg color
        let cursor_line = &styled[0];
        assert_eq!(
            cursor_line.width(),
            usize::from(vp.width),
            "line without ANSI content should still fill viewport width"
        );
    }

    #[test]
    fn cursor_at_filename_start_with_prefix_column() {
        let mut vp = directory_current_viewport(40, 10);
        vp.prefix_column_width = 2;
        // Cursor at horizontal index 0 (first char of filename)
        vp.cursor = Cursor {
            vertical_index: 0,
            horizontal_index: CursorPosition::Absolute {
                current: 0,
                expanded: 0,
            },
        };
        let lines = vec![BufferLine {
            prefix: Some("\u{f0f6}".to_string()),
            ..BufferLine::from("documents")
        }];

        let styled = get_styled_lines(&vp, &Mode::Normal, &vp.cursor, lines, &test_theme());

        assert!(!styled.is_empty());
        let cursor_line = &styled[0];
        // The line should still fill the expected width (viewport - border)
        let expected_width = usize::from(vp.width) - 1;
        assert_eq!(
            cursor_line.width(),
            expected_width,
            "cursor line with prefix column should fill expected width"
        );
        // Cursor at position 0 should highlight the 'd' in "documents",
        // not the prefix glyph (prefix is in the prefix column, not in content).
        // We verify by checking the cursor styling appears in the spans.
        let has_cursor_style = cursor_line
            .spans
            .iter()
            .any(|span| span.content.contains('d') && span.style.bg.is_some());
        assert!(
            has_cursor_style,
            "cursor should be on the first content character, not on the prefix column"
        );
    }

    #[test]
    fn prefix_column_icon_not_inheriting_line_number_bold() {
        use ratatui::style::Modifier;

        let mut vp = directory_current_viewport(40, 10);
        vp.prefix_column_width = 2;
        vp.line_number = LineNumber::Absolute;
        let lines = vec![
            BufferLine {
                prefix: Some("\u{f0f6}".to_string()),
                ..BufferLine::from("documents")
            },
            BufferLine {
                prefix: Some("\u{f0f6}".to_string()),
                ..BufferLine::from("pictures")
            },
        ];

        let styled = get_styled_lines(&vp, &Mode::Navigation, &vp.cursor, lines, &test_theme());

        assert!(styled.len() >= 2);
        let non_cursor_line = &styled[1];
        for span in &non_cursor_line.spans {
            if span.content.contains('\u{f0f6}') {
                assert!(
                    !span.style.add_modifier.contains(Modifier::BOLD),
                    "prefix icon span should not inherit BOLD from line number, got style: {:?}",
                    span.style,
                );
            }
        }
    }
}
