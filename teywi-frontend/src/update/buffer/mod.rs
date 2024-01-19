use teywi_keymap::action::Action;

use crate::model::{Buffer, CursorPosition};

pub fn update(model: &mut Buffer, message: &Action) {
    match message {
        Action::KeySequenceChanged(_) => {}
        Action::ModeChanged(_) => {}
        Action::MoveCursorDown => {
            if model.lines.len() - 1 > model.cursor.line_number {
                model.cursor.line_number += 1;
            }
        }
        Action::MoveCursorRight => {
            let cursor_index = match model.cursor.horizontial_position {
                CursorPosition::Absolute(n) => n,
                CursorPosition::_End => return,
            };

            if model.lines[model.cursor.line_number].chars().count() - 1 > cursor_index {
                model.cursor.horizontial_position = CursorPosition::Absolute(cursor_index + 1);
            }
        }
        Action::Refresh => {}
        Action::Quit => {}
    }
}
