use crate::{
    ansi,
    model::{viewport::ViewPort, BufferLine, Cursor, CursorPosition, Mode},
};

pub fn add_cursor_styles(
    vp: &ViewPort,
    mode: &Mode,
    cursor: &Option<Cursor>,
    index: &usize,
    line: &mut BufferLine,
) -> String {
    if let Some(cursor) = cursor {
        if cursor.vertical_index - vp.vertical_index != *index {
            return line.content.clone();
        }

        let content_width = vp.get_content_width(line);
        let horizontal_index = match ansi::get_index_for_char(&line.content, vp.horizontal_index) {
            Some(i) => i,
            None => 0,
        };

        let mut content = format!(
            "{}{}",
            ansi::get_ansi_escape_sequences_till_char(&line.content, vp.horizontal_index),
            &line.content[horizontal_index..]
        );

        let char_count = ansi::get_char_count(&content);
        let line_length = if char_count > content_width {
            content_width
        } else if char_count == 0 {
            1
        } else {
            char_count
        };

        let repeat_count = if content_width > line_length {
            content_width - line_length
        } else {
            0
        };
        content = if cursor.hide_cursor_line {
            format!("{}{}", content, " ".repeat(repeat_count))
        } else {
            format!("\x1b[100m{}{}\x1b[0m", content, " ".repeat(repeat_count))
        };

        if cursor.hide_cursor {
            return content;
        }

        let cursor_on_char_count = match &cursor.horizontal_index {
            CursorPosition::End => line_length,
            CursorPosition::None => return content,
            CursorPosition::Absolute {
                current,
                expanded: _,
            } => *current + 1,
        };

        let cursor_index = match ansi::get_index_for_char(&content, cursor_on_char_count) {
            Some(i) => i,
            None => return content,
        };

        // FIX: reset should just use the ansi code for reset inverse (27)
        // https://github.com/ratatui/ansi-to-tui/issues/50
        let reset = format!(
            "\x1b[0m{}",
            ansi::get_ansi_escape_sequences_till_char(&content, cursor_on_char_count)
        );

        let (code, reset) = match mode {
            Mode::Command(_) | Mode::Normal => ("\x1b[7m", reset.as_str()),
            Mode::Insert => ("\x1b[4m", reset.as_str()),
            Mode::Navigation => ("", ""),
        };

        let content = format!(
            "{}{}{}{}{}",
            &content[..cursor_index],
            code,
            &content[cursor_index..cursor_index + 1],
            reset,
            &content[cursor_index + 1..]
        );

        content
    } else {
        line.content.clone()
    }
}
