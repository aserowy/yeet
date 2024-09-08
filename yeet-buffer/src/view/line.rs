use crate::{
    ansi,
    model::{viewport::ViewPort, BufferLine, Cursor, CursorPosition, Mode},
};

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

        // TODO: slice function which preserves ansi codes
        let content = &line.content[vp.horizontal_index..];
        let char_count = ansi::get_char_count(content);

        let line_length = if char_count > content_width {
            content_width
        } else if char_count == 0 {
            1
        } else {
            char_count
        };

        let mut content = line.content.clone();
        if !cursor.hide_cursor_line {
            content = format!("\x1b[100m{}\x1b[0m", content);
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

        content
    } else {
        line.content.clone()
    }
}
