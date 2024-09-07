use crate::model::{viewport::ViewPort, BufferLine, Cursor, CursorPosition, Mode};

pub fn add_cursor_styles(
    vp: &ViewPort,
    _mode: &Mode,
    cursor: &Option<Cursor>,
    index: &usize,
    line: &mut BufferLine,
) -> String {
    if let Some(cursor) = cursor {
        if cursor.vertical_index - vp.vertical_index != *index {
            return line.content.clone();
        }

        let content_width = vp.get_content_width(line);

        let content = &line.content[vp.horizontal_index..];
        let chars_count = content.chars().count();

        let line_length = if chars_count > content_width {
            content_width
        } else if chars_count == 0 {
            1
        } else {
            chars_count
        };

        let mut content = line.content.clone();
        if !cursor.hide_cursor_line {
            content = format!("\x1b[1;100m{}\x1b[0m", content);
        }

        if cursor.hide_cursor {
            return content;
        }

        let _cursor_index = match &cursor.horizontal_index {
            CursorPosition::End => line_length - vp.horizontal_index - 1,
            CursorPosition::None => return content,
            CursorPosition::Absolute {
                current,
                expanded: _,
            } => *current - vp.horizontal_index,
        };

        // spans.push(StylePartialSpan {
        //     start: offset + cursor_index,
        //     end: offset + cursor_index + 1,
        //     style: get_cursor_partial_style(mode),
        // });
        //

        return content;
    }
    return line.content.clone();
}

// fn get_cursorline_partial_style(mode: &Mode) -> StylePartial {
//     match mode {
//         Mode::Command(_) => CURSORLINE_NAV_STYLE_PARTIAL.clone(),
//         Mode::Insert => CURSORLINE_NORMAL_STYLE_PARTIAL.clone(),
//         Mode::Navigation => CURSORLINE_NAV_STYLE_PARTIAL.clone(),
//         Mode::Normal => CURSORLINE_NORMAL_STYLE_PARTIAL.clone(),
//     }
// }
//
// fn get_cursor_partial_style(mode: &Mode) -> StylePartial {
//     match mode {
//         Mode::Command(_) => CURSOR_COMMAND_STYLE_PARTIAL.clone(),
//         Mode::Insert => CURSOR_INSERT_STYLE_PARTIAL.clone(),
//         Mode::Navigation => CURSOR_NAV_STYLE_PARTIAL.clone(),
//         Mode::Normal => CURSOR_NORMAL_STYLE_PARTIAL.clone(),
//     }
// }
