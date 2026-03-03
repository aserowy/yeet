use crate::{
    message::CursorDirection,
    model::{BufferLine, Cursor, CursorPosition, TextBuffer},
};

use super::cursor;

pub fn char(direction: &CursorDirection, cursor: &mut Cursor, model: &mut TextBuffer) {
    match direction {
        CursorDirection::FindBackward(find) => {
            if let Some(found) = find_char_backward(find, &model.lines, cursor) {
                cursor.horizontal_index = CursorPosition::Absolute {
                    current: found,
                    expanded: found,
                };
            }
        }
        CursorDirection::FindForward(find) => {
            if let Some(found) = find_char_forward(find, &model.lines, cursor) {
                cursor.horizontal_index = CursorPosition::Absolute {
                    current: found,
                    expanded: found,
                };
            }
        }
        CursorDirection::TillBackward(find) => {
            if let Some(found) = find_char_backward(find, &model.lines, cursor) {
                let new = found + 1;
                cursor.horizontal_index = CursorPosition::Absolute {
                    current: new,
                    expanded: new,
                };
            }
        }
        CursorDirection::TillForward(find) => {
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
}

fn find_char_backward(find: &char, lines: &[BufferLine], cursor: &Cursor) -> Option<usize> {
    let current = lines.get(cursor.vertical_index)?;
    let index = cursor::get_horizontal_index(&cursor.horizontal_index, current)?;

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

fn find_char_forward(find: &char, lines: &[BufferLine], cursor: &Cursor) -> Option<usize> {
    let current = lines.get(cursor.vertical_index)?;
    let index = cursor::get_horizontal_index(&cursor.horizontal_index, current)?;

    current
        .content
        .to_stripped_string()
        .chars()
        .skip(index + 1)
        .position(|c| &c == find)
        .map(|i| index + i + 1)
}
