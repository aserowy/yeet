use ansi_to_tui::IntoText;
use ratatui::{
    prelude::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{BufferTheme, model::{ansi::Ansi, viewport::ViewPort, BufferLine, Cursor, Mode, TextBuffer}};

mod line;
mod prefix;

pub fn view(viewport: &ViewPort, mode: &Mode, buffer: &TextBuffer, theme: &BufferTheme, frame: &mut Frame) {
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
            .border_style(Style::default().fg(Color::Black));

        let inner = block.inner(rect);

        frame.render_widget(block, rect);

        inner
    } else {
        rect
    };

    frame.render_widget(Paragraph::new(styled), rect);
}

fn get_rendered_lines(viewport: &ViewPort, buffer: &TextBuffer) -> Vec<BufferLine> {
    buffer
        .lines
        .iter()
        .skip(viewport.vertical_index)
        .take(usize::from(viewport.height))
        .map(|line| line.to_owned())
        .collect()
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

    let mut result = Vec::new();
    for (i, mut bl) in lines.into_iter().enumerate() {
        let corrected_index = i + vp.vertical_index;

        let content = Ansi::new("")
            .join(&prefix::get_signs(vp, &bl))
            .join(&prefix::get_line_number(vp, corrected_index, cursor, theme))
            .join(&prefix::get_custom_prefix(&bl))
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
    use crate::{BufferTheme, model::{
        viewport::{LineNumber, ViewPort},
        BufferLine, Cursor, CursorPosition, Mode,
    }};

    use super::get_styled_lines;

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

        let styled = get_styled_lines(&vp, &Mode::Navigation, &vp.cursor, lines, &BufferTheme::default());

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

        let styled = get_styled_lines(&vp, &Mode::Navigation, &vp.cursor, lines, &BufferTheme::default());

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

        let styled = get_styled_lines(&vp, &Mode::Navigation, &vp.cursor, lines, &BufferTheme::default());

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

        let styled = get_styled_lines(&vp, &Mode::Navigation, &vp.cursor, lines, &BufferTheme::default());

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

        let styled = get_styled_lines(&vp, &Mode::Navigation, &vp.cursor, lines, &BufferTheme::default());

        assert!(!styled.is_empty());
        let cursor_line = &styled[0];
        assert_eq!(
            cursor_line.width(),
            usize::from(vp.width),
            "cursor line width with ANSI-styled content should equal viewport width"
        );
    }

    #[test]
    fn cursor_line_width_in_normal_mode() {
        let vp = tasks_viewport(80, 10);
        let lines = vec![BufferLine::from("1    rg foo")];

        let styled = get_styled_lines(&vp, &Mode::Normal, &vp.cursor, lines, &BufferTheme::default());

        assert!(!styled.is_empty());
        let cursor_line = &styled[0];
        assert_eq!(
            cursor_line.width(),
            usize::from(vp.width),
            "cursor line width in Normal mode should equal viewport width"
        );
    }
}
