use yate_keymap::message::{CursorDirection, Message, Mode};

use crate::model::buffer::{
    viewport::ViewPort, Buffer, BufferLine, BufferResult, Cursor, CursorPosition,
};

mod bufferline;
pub mod cursor;
pub mod viewport;

pub fn update(mode: &Mode, model: &mut Buffer, message: &Message) -> Option<BufferResult> {
    // TODO: split message and buffer related messages
    let result = match message {
        Message::ChangeMode(from, to) => {
            if from == &Mode::Insert && to != &Mode::Insert {
                model.undo.close_transaction();
                cursor::update_by_direction(mode, model, &1, &CursorDirection::Left);
            }
            None
        }
        Message::Modification(modification) => {
            let buffer_changes = bufferline::update(model, modification);
            if let Some(changes) = buffer_changes {
                model.undo.add(mode, changes);
            }
            None
        }
        Message::MoveCursor(count, direction) => {
            cursor::update_by_direction(mode, model, count, direction);
            None
        }
        Message::MoveViewPort(direction) => {
            viewport::update_by_direction(model, direction);
            None
        }
        Message::SaveBuffer(_) => {
            let changes = model.undo.save();
            Some(BufferResult::Changes(changes))
        }
        Message::SelectCurrent | Message::SelectParent => {
            reset_view(&mut model.view_port, &mut model.cursor);
            None
        }
        Message::ChangeKeySequence(_)
        | Message::ExecuteCommand
        | Message::PathAdded(_)
        | Message::PathRemoved(_)
        | Message::Refresh
        | Message::SelectPath(_)
        | Message::Quit => None,
    };

    viewport::update_by_cursor(model);

    result
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

pub fn set_content(mode: &Mode, model: &mut Buffer, content: Vec<BufferLine>) {
    model.lines = content;
    cursor::validate(mode, model);
}

pub fn unfocus_buffer(buffer: &mut Buffer) {
    if let Some(cursor) = &mut buffer.cursor {
        cursor.hide_cursor = true;
    }
}
