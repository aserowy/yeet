use yate_keymap::action::CursorDirection;

use crate::model::buffer::{Buffer, CursorPosition};

use super::viewport;

pub fn update(model: &mut Buffer, direction: &CursorDirection) {
    match direction {
        CursorDirection::Bottom => {
            model.cursor.vertical_index = model.lines.len() - 1;
        }
        CursorDirection::Down => {
            if model.lines.len() - 1 > model.cursor.vertical_index {
                model.cursor.vertical_index += 1;
            }
        }
        CursorDirection::Left => {
            let cursor_index = match model.cursor.horizontial_index {
                CursorPosition::Absolute(n) => n,
                CursorPosition::End => model.lines[model.cursor.vertical_index].chars().count() - 1,
            };

            if cursor_index > 0 {
                model.cursor.horizontial_index = CursorPosition::Absolute(cursor_index - 1);
            }
        }
        CursorDirection::LineEnd => {
            model.cursor.horizontial_index = CursorPosition::End;
        }
        CursorDirection::LineStart => {
            model.cursor.horizontial_index = CursorPosition::Absolute(0);
        }
        CursorDirection::Right => {
            let cursor_index = match model.cursor.horizontial_index {
                CursorPosition::Absolute(n) => n,
                CursorPosition::End => return,
            };

            if model.lines[model.cursor.vertical_index].chars().count() - 1 > cursor_index {
                model.cursor.horizontial_index = CursorPosition::Absolute(cursor_index + 1);
            }
        }
        CursorDirection::Top => {
            model.cursor.vertical_index = 0;
        }
        CursorDirection::Up => {
            if model.cursor.vertical_index > 0 {
                model.cursor.vertical_index -= 1;
            }
        }
    }

    viewport::update_by_cursor(model);
}
