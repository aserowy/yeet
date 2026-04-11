use ratatui::style::Color;

use crate::{
    model::{ansi::Ansi, viewport::ViewPort, BufferLine, Cursor, CursorPosition, Mode},
    BufferTheme,
};

use super::style::{
    self, CURSOR_INSERT_CODE, CURSOR_INSERT_RESET, CURSOR_NORMAL_CODE, CURSOR_NORMAL_RESET,
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

    let cursor_line_offset = cursor.vertical_index.checked_sub(vp.vertical_index);
    let is_cursor_line = cursor_line_offset == Some(*index);
    let use_cursor_line_bg = is_cursor_line && !vp.hide_cursor_line;

    let bg = if use_cursor_line_bg {
        theme.cursor_line_bg
    } else {
        theme.buffer_bg
    };

    let ansi = add_search_styles(line, &ansi, theme.search_bg, bg);

    if !is_cursor_line {
        if cursor_line_offset.is_none() {
            return ansi;
        }
        let buffer_bg = style::color_to_ansi_bg(theme.buffer_bg);
        let reset_with_bg = format!("\x1b[0m{}", buffer_bg);
        let mut result = ansi;
        result.replace_resets_with(&reset_with_bg);
        result.prepend(&buffer_bg);
        result
    } else {
        add_cursor_styles(vp, mode, cursor, content_width, &ansi, theme)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn add_line_styles_wrap(
    vp: &ViewPort,
    mode: &Mode,
    cursor: &Cursor,
    index: &usize,
    line: &mut BufferLine,
    theme: &BufferTheme,
    content_width: usize,
    char_offset: usize,
) -> Ansi {
    let ansi = line.content.clone();

    let cursor_line_offset = cursor.vertical_index.checked_sub(vp.vertical_index);
    let is_cursor_line = cursor_line_offset == Some(*index);
    let use_cursor_line_bg = is_cursor_line && !vp.hide_cursor_line;

    let bg = if use_cursor_line_bg {
        theme.cursor_line_bg
    } else {
        theme.buffer_bg
    };

    let ansi = add_search_styles(line, &ansi, theme.search_bg, bg);

    if !is_cursor_line {
        if cursor_line_offset.is_none() {
            return ansi;
        }
        let buffer_bg = style::color_to_ansi_bg(theme.buffer_bg);
        let reset_with_bg = format!("\x1b[0m{}", buffer_bg);
        let mut result = ansi;
        result.replace_resets_with(&reset_with_bg);
        result.prepend(&buffer_bg);
        result
    } else {
        add_cursor_styles_wrap(vp, mode, cursor, content_width, &ansi, theme, char_offset)
    }
}

fn add_cursor_styles_wrap(
    vp: &ViewPort,
    mode: &Mode,
    cursor: &Cursor,
    content_width: usize,
    ansi: &Ansi,
    theme: &BufferTheme,
    char_offset: usize,
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
        let buffer_bg = style::color_to_ansi_bg(theme.buffer_bg);
        let reset_with_bg = format!("\x1b[0m{}", buffer_bg);
        content.replace_resets_with(&reset_with_bg);
        content.append(" ".repeat(repeat_count).as_str());
    } else {
        let cursor_line_bg = style::color_to_ansi_bg(theme.cursor_line_bg);
        let cursor_line_reset = &style::ansi_reset_with_bg(theme.buffer_bg);
        let reset_with_bg = format!("\x1b[0m{}", cursor_line_bg);
        content.replace_resets_with(&reset_with_bg);
        content.prepend(&cursor_line_bg);
        content.append(" ".repeat(repeat_count).as_str());
        content.append(cursor_line_reset);
    };

    if vp.hide_cursor {
        return content;
    }

    let cursor_abs = match &cursor.horizontal_index {
        CursorPosition::End => {
            return content;
        }
        CursorPosition::None => return content,
        CursorPosition::Absolute {
            current,
            expanded: _,
        } => *current,
    };

    let segment_end = char_offset + char_count;
    if cursor_abs < char_offset || cursor_abs >= segment_end {
        return content;
    }

    let cursor_index = cursor_abs - char_offset;

    let (code, reset) = match mode {
        Mode::Command(_) | Mode::Normal => (CURSOR_NORMAL_CODE, CURSOR_NORMAL_RESET),
        Mode::Insert => (CURSOR_INSERT_CODE, CURSOR_INSERT_RESET),
        Mode::Navigation => ("", ""),
    };

    content.insert(cursor_index, code);
    content.insert(cursor_index + 1, reset);

    content
}

fn add_search_styles(line: &BufferLine, ansi: &Ansi, search: Color, bg: Color) -> Ansi {
    if let Some(search_char_position) = &line.search_char_position {
        let search_bg = style::color_to_ansi_bg(search);
        let bg_reset = style::ansi_reset_with_bg(bg);
        let mut content = ansi.clone();
        for (index, length) in search_char_position.iter() {
            let reset = format!(
                "{}{}",
                bg_reset,
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
        let buffer_bg = style::color_to_ansi_bg(theme.buffer_bg);
        let reset_with_bg = format!("\x1b[0m{}", buffer_bg);
        content.replace_resets_with(&reset_with_bg);
        content.append(" ".repeat(repeat_count).as_str());
    } else {
        let cursor_line_bg = style::color_to_ansi_bg(theme.cursor_line_bg);
        let cursor_line_reset = &style::ansi_reset_with_bg(theme.buffer_bg);
        let reset_with_bg = format!("\x1b[0m{}", cursor_line_bg);
        content.replace_resets_with(&reset_with_bg);
        content.prepend(&cursor_line_bg);
        content.append(" ".repeat(repeat_count).as_str());
        content.append(cursor_line_reset);
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
