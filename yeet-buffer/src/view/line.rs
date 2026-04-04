use crate::{
    model::{ansi::Ansi, viewport::ViewPort, BufferLine, Cursor, CursorPosition, Mode},
    BufferTheme,
};

use super::style::{
    self, CURSOR_INSERT_CODE, CURSOR_INSERT_RESET, CURSOR_LINE_RESET, CURSOR_NORMAL_CODE,
    CURSOR_NORMAL_RESET,
};

pub fn add_line_styles(
    vp: &ViewPort,
    mode: &Mode,
    cursor: &Cursor,
    index: &usize,
    line: &mut BufferLine,
    theme: &BufferTheme,
) -> Ansi {
    let content_width = vp.get_content_width(line);
    let ansi = line.content.skip_chars(vp.horizontal_index);
    let ansi = add_search_styles(line, &ansi, theme);

    let cursor_line_offset = match cursor.vertical_index.checked_sub(vp.vertical_index) {
        Some(offset) => offset,
        None => return ansi,
    };

    if cursor_line_offset != *index {
        ansi
    } else {
        add_cursor_styles(vp, mode, cursor, content_width, &ansi, theme)
    }
}

fn add_search_styles(line: &BufferLine, ansi: &Ansi, theme: &BufferTheme) -> Ansi {
    if let Some(search_char_position) = &line.search_char_position {
        let search_bg = style::color_to_ansi_bg(theme.search_bg);
        let mut content = ansi.clone();
        for (index, length) in search_char_position.iter() {
            let reset = format!(
                "\x1b[0m{}",
                content.get_ansi_escape_sequences_till_char(*index + 1)
            );

            content.insert(*index, &search_bg);
            content.insert(index + length, &reset);
        }
        content
    } else {
        ansi.clone()
    }
}

fn add_cursor_styles(
    vp: &ViewPort,
    mode: &Mode,
    cursor: &Cursor,
    content_width: usize,
    ansi: &Ansi,
    theme: &BufferTheme,
) -> Ansi {
    let mut content = ansi.clone();
    let char_count = content.count_chars();
    let line_length = if char_count > content_width {
        content_width
    } else {
        char_count
    };

    let repeat_count = content_width.saturating_sub(line_length);
    if vp.hide_cursor_line {
        content.append(" ".repeat(repeat_count).as_str());
    } else {
        let cursor_line_bg = style::color_to_ansi_bg(theme.cursor_line_bg);
        content.prepend(&cursor_line_bg);
        content.append(" ".repeat(repeat_count).as_str());
        content.append(CURSOR_LINE_RESET);
    };

    if vp.hide_cursor {
        return content;
    }

    let cursor_index = match &cursor.horizontal_index {
        CursorPosition::End => char_count.saturating_sub(1),
        CursorPosition::None => return content,
        CursorPosition::Absolute {
            current,
            expanded: _,
        } => current.saturating_sub(vp.horizontal_index),
    };

    let (code, reset) = match mode {
        Mode::Command(_) | Mode::Normal => (CURSOR_NORMAL_CODE, CURSOR_NORMAL_RESET),
        Mode::Insert => (CURSOR_INSERT_CODE, CURSOR_INSERT_RESET),
        Mode::Navigation => ("", ""),
    };

    content.insert(cursor_index, code);
    content.insert(cursor_index + 1, reset);

    content
}
