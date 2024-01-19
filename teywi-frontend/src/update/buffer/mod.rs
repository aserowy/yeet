use teywi_keymap::action::{Action, Direction};

use crate::model::{Buffer, CursorPosition};

pub fn update(model: &mut Buffer, message: &Action) {
    match message {
        Action::ChangeKeySequence(_) => {}
        Action::ChangeMode(_) => {}
        Action::MoveCursor(direction) => match direction {
            Direction::Bottom => {
                model.cursor.line_number = model.lines.len() - 1;
            }
            Direction::Down => {
                if model.lines.len() - 1 > model.cursor.line_number {
                    model.cursor.line_number += 1;
                }
            }
            Direction::Left => {
                let cursor_index = match model.cursor.horizontial_position {
                    CursorPosition::Absolute(n) => n,
                    CursorPosition::End => {
                        model.lines[model.cursor.line_number].chars().count() - 1
                    }
                };

                if cursor_index > 0 {
                    model.cursor.horizontial_position = CursorPosition::Absolute(cursor_index - 1);
                }
            }
            Direction::LineEnd => {
                model.cursor.horizontial_position = CursorPosition::End;
            }
            Direction::LineStart => {
                model.cursor.horizontial_position = CursorPosition::Absolute(0);
            }
            Direction::Right => {
                let cursor_index = match model.cursor.horizontial_position {
                    CursorPosition::Absolute(n) => n,
                    CursorPosition::End => return,
                };

                if model.lines[model.cursor.line_number].chars().count() - 1 > cursor_index {
                    model.cursor.horizontial_position = CursorPosition::Absolute(cursor_index + 1);
                }
            }
            Direction::Top => {
                model.cursor.line_number = 0;
            }
            Direction::Up => {
                if model.cursor.line_number > 0 {
                    model.cursor.line_number -= 1;
                }
            }
        },
        Action::Refresh => {}
        Action::Quit => {}
    }
}
