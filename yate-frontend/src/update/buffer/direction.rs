use yate_keymap::message::CursorDirection;

use crate::model::buffer::{Buffer, CursorPosition};

use super::viewport;

pub fn update(model: &mut Buffer, count: &usize, direction: &CursorDirection) {
    if let Some(cursor) = &mut model.cursor {
        for _ in 0..*count {
            match direction {
                CursorDirection::Bottom => {
                    cursor.vertical_index = model.lines.len() - 1;
                }
                CursorDirection::Down => {
                    if model.lines.len() - 1 > cursor.vertical_index {
                        cursor.vertical_index += 1;
                    }
                }
                CursorDirection::Left => {
                    let cursor_index = match cursor.horizontial_index {
                        CursorPosition::Absolute(n) => n,
                        CursorPosition::End => {
                            model.lines[cursor.vertical_index].chars().count() - 1
                        }
                        CursorPosition::None => return,
                    };

                    if cursor_index > 0 {
                        cursor.horizontial_index = CursorPosition::Absolute(cursor_index - 1);
                    }
                }
                CursorDirection::LineEnd => {
                    cursor.horizontial_index = CursorPosition::End;
                }
                CursorDirection::LineStart => {
                    cursor.horizontial_index = CursorPosition::Absolute(0);
                }
                CursorDirection::Right => {
                    let cursor_index = match cursor.horizontial_index {
                        CursorPosition::Absolute(n) => n,
                        CursorPosition::End => return,
                        CursorPosition::None => return,
                    };

                    if model.lines[cursor.vertical_index].chars().count() - 1 > cursor_index {
                        cursor.horizontial_index = CursorPosition::Absolute(cursor_index + 1);
                    }
                }
                CursorDirection::Top => {
                    cursor.vertical_index = 0;
                }
                CursorDirection::Up => {
                    if cursor.vertical_index > 0 {
                        cursor.vertical_index -= 1;
                    }
                }
            }
        }

        viewport::update_by_cursor(model);
    }
}
