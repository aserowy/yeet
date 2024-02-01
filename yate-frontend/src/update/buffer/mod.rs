use yate_keymap::message::{Message, Mode};

use crate::model::buffer::{Buffer, BufferChanged, Cursor, CursorPosition, ViewPort};

mod bufferline;
mod cursor;
pub mod viewport;

pub fn update(mode: &Mode, model: &mut Buffer, message: &Message) -> Option<Vec<BufferChanged>> {
    match message {
        Message::ChangeKeySequence(_) => None,
        Message::ChangeMode(_, _) => None,
        Message::ExecuteCommand => None,
        Message::Modification(modification) => bufferline::update(model, modification),
        Message::MoveCursor(count, direction) => {
            cursor::update_by_direction(mode, model, count, direction);
            viewport::update_by_cursor(model);

            None
        }
        Message::MoveViewPort(direction) => {
            viewport::update_by_direction(model, direction);

            None
        }
        Message::Refresh => None,
        Message::SelectCurrent => {
            reset_view(&mut model.view_port, &mut model.cursor);

            None
        }
        Message::SelectParent => {
            reset_view(&mut model.view_port, &mut model.cursor);

            None
        }
        Message::Quit => None,
    }
}

pub fn focus_buffer(buffer: &mut Buffer) {
    if let Some(cursor) = &mut buffer.cursor {
        cursor.hide_cursor = false;
    }
}

pub fn reset_view(view_port: &mut ViewPort, cursor: &mut Option<Cursor>) {
    view_port.horizontal_index = 0;
    view_port.vertical_index = 0;

    if let Some(cursor) = cursor {
        cursor.vertical_index = 0;

        cursor.horizontial_index = match &cursor.horizontial_index {
            CursorPosition::Absolute {
                current: _,
                expanded: _,
            } => CursorPosition::Absolute {
                current: 0,
                expanded: 0,
            },
            CursorPosition::End => CursorPosition::End,
            CursorPosition::None => CursorPosition::None,
        }
    }
}

pub fn unfocus_buffer(buffer: &mut Buffer) {
    if let Some(cursor) = &mut buffer.cursor {
        cursor.hide_cursor = true;
    }
}
