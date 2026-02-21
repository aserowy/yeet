use crate::model::{ansi::Ansi, viewport::ViewPort, BufferLine, Cursor, CursorPosition, Mode};

pub fn add_line_styles(
    vp: &ViewPort,
    mode: &Mode,
    cursor: &Cursor,
    index: &usize,
    line: &mut BufferLine,
) -> Ansi {
    let content_width = vp.get_content_width(line);
    let ansi = line.content.skip_chars(vp.horizontal_index);
    let ansi = add_search_styles(line, &ansi);

    if cursor.vertical_index - vp.vertical_index != *index {
        ansi
    } else {
        add_cursor_styles(vp, mode, cursor, content_width, &ansi)
    }
}

fn add_search_styles(line: &BufferLine, ansi: &Ansi) -> Ansi {
    if let Some(search_char_position) = &line.search_char_position {
        let mut content = ansi.clone();
        for (index, length) in search_char_position.iter() {
            let reset = format!(
                "\x1b[0m{}",
                content.get_ansi_escape_sequences_till_char(*index + 1)
            );

            content.insert(*index, "\x1b[41m");
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
) -> Ansi {
    let mut content = ansi.clone();
    let char_count = content.count_chars();
    let line_length = if char_count > content_width {
        content_width
    } else if char_count == 0 {
        1
    } else {
        char_count
    };

    let repeat_count = content_width.saturating_sub(line_length);
    if vp.hide_cursor_line {
        content.append(" ".repeat(repeat_count).as_str());
    } else {
        content.prepend("\x1b[100m");
        content.append(" ".repeat(repeat_count).as_str());
        content.append("\x1b[0m");
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

    // FIX: reset should just use the ansi code for reset inverse (27)
    // https://github.com/ratatui/ansi-to-tui/issues/50
    let reset = format!(
        "\x1b[0m{}",
        content.get_ansi_escape_sequences_till_char(cursor_index + 1)
    );

    let (code, reset) = match mode {
        Mode::Command(_) | Mode::Normal => ("\x1b[7m", reset.as_str()),
        Mode::Insert => ("\x1b[4m", reset.as_str()),
        Mode::Navigation => ("", ""),
    };

    content.insert(cursor_index, code);
    content.insert(cursor_index + 1, reset);

    content
}
