use crate::{
    message::CursorDirection,
    model::{Buffer, BufferLine, Cursor, CursorPosition},
};

use super::cursor;

pub fn char(direction: &CursorDirection, model: &mut Buffer, set_last_find: bool) {
    match direction {
        CursorDirection::FindBackward(find) => {
            let cursor = match &mut model.cursor {
                Some(cursor) => cursor,
                None => return,
            };

            if let Some(found) = find_char_backward(find, &model.lines, cursor) {
                cursor.horizontal_index = CursorPosition::Absolute {
                    current: found,
                    expanded: found,
                };
            }
        }
        CursorDirection::FindForward(find) => {
            let cursor = match &mut model.cursor {
                Some(cursor) => cursor,
                None => return,
            };

            if let Some(found) = find_char_forward(find, &model.lines, cursor) {
                cursor.horizontal_index = CursorPosition::Absolute {
                    current: found,
                    expanded: found,
                };
            }
        }
        CursorDirection::TillBackward(find) => {
            let cursor = match &mut model.cursor {
                Some(cursor) => cursor,
                None => return,
            };

            if let Some(found) = find_char_backward(find, &model.lines, cursor) {
                let new = found + 1;
                cursor.horizontal_index = CursorPosition::Absolute {
                    current: new,
                    expanded: new,
                };
            }
        }
        CursorDirection::TillForward(find) => {
            let cursor = match &mut model.cursor {
                Some(cursor) => cursor,
                None => return,
            };

            if let Some(found) = find_char_forward(find, &model.lines, cursor) {
                let new = found - 1;
                cursor.horizontal_index = CursorPosition::Absolute {
                    current: new,
                    expanded: new,
                };
            }
        }
        _ => unreachable!(),
    };

    if set_last_find {
        model.last_find = Some(direction.clone());
    }
}

fn find_char_backward(find: &char, lines: &[BufferLine], cursor: &Cursor) -> Option<usize> {
    let current = match lines.get(cursor.vertical_index) {
        Some(line) => line,
        None => return None,
    };

    let index = match cursor::get_horizontal_index(&cursor.horizontal_index, current) {
        Some(index) => index,
        None => return None,
    };

    if index <= 1 {
        return None;
    }

    current
        .content
        .to_stripped_string()
        .chars()
        .take(index)
        .collect::<Vec<_>>()
        .iter()
        .rposition(|c| c == find)
}

fn find_char_forward(find: &char, lines: &[BufferLine], cursor: &mut Cursor) -> Option<usize> {
    let current = match lines.get(cursor.vertical_index) {
        Some(line) => line,
        None => return None,
    };

    let index = match cursor::get_horizontal_index(&cursor.horizontal_index, current) {
        Some(index) => index,
        None => return None,
    };

    current
        .content
        .to_stripped_string()
        .chars()
        .skip(index + 1)
        .position(|c| &c == find)
        .map(|i| index + i + 1)
}
