use yate_keymap::message::TextModification;

use crate::model::buffer::{Buffer, BufferChanged, BufferLine, Cursor, CursorPosition};

pub fn update(model: &mut Buffer, modification: &TextModification) -> Option<Vec<BufferChanged>> {
    // TODO: most None must return Some(Vec<BufferChanged>) instead
    match modification {
        // TODO: add delete char before cursor for <bs> and use this for x
        // TODO: remove visual cursor offset
        TextModification::DeleteCharOnCursor => {
            let line = get_line(model);
            if let Some((cursor, line)) = line {
                let index = get_cursor_index(cursor, line);
                if index > 0 {
                    cursor.horizontial_index = CursorPosition::Absolute(index - 1);

                    line.content =
                        format!("{}{}", &line.content[..index - 1], &line.content[index..]);
                    None
                } else {
                    // TODO: if insert mode, delete line
                    None
                }
            } else {
                None
            }
        }
        TextModification::DeleteLineOnCursor => {
            if model.lines.is_empty() {
                None
            } else if let Some(cursor) = &mut model.cursor {
                let line_index = cursor.vertical_index;
                let content = model.lines.remove(line_index).content.to_string();

                Some(vec![BufferChanged::LineDeleted(line_index, content)])
            } else {
                None
            }
        }
        TextModification::Insert(raw) => {
            let line = get_line(model);
            if let Some((cursor, line)) = line {
                let index = get_cursor_index(cursor, line);

                cursor.horizontial_index = CursorPosition::Absolute(index + raw.chars().count());

                line.content = format!(
                    "{}{}{}",
                    &line.content[..index],
                    raw,
                    &line.content[index..]
                );
            }

            None
        }
    }
}

fn get_line<'a>(model: &'a mut Buffer) -> Option<(&'a mut Cursor, &'a mut BufferLine)> {
    if let Some(cursor) = &mut model.cursor {
        if cursor.horizontial_index == CursorPosition::None {
            return None;
        }

        if model.lines.is_empty() {
            cursor.vertical_index = 0;

            let line = BufferLine::default();
            model.lines.push(line);

            Some((cursor, &mut model.lines[0]))
        } else {
            let line_index = cursor.vertical_index;

            Some((cursor, &mut model.lines[line_index]))
        }
    } else {
        None
    }
}

fn get_cursor_index(cursor: &Cursor, line: &BufferLine) -> usize {
    match cursor.horizontial_index {
        CursorPosition::Absolute(n) => n,
        // FIX: count > 0 checks
        CursorPosition::End => line.content.chars().count() - 1,
        CursorPosition::None => unreachable!(),
    }
}
