use crate::{
    message::{BufferMessage, CursorDirection},
    model::{Buffer, BufferLine, BufferResult, CursorPosition, Mode, SearchModel},
};

mod bufferline;
pub mod cursor;
pub mod viewport;

pub fn update(
    mode: &Mode,
    search: &Option<SearchModel>,
    model: &mut Buffer,
    message: &BufferMessage,
) -> Option<BufferResult> {
    let result = match message {
        // TODO: repeat actions by count when switching from insert to normal
        // count is entered before going into insert. ChangeMode with count? Or Insert with count?
        BufferMessage::ChangeMode(from, to) => {
            if from == &Mode::Insert && to != &Mode::Insert {
                model.undo.close_transaction();
                cursor::update_by_direction(mode, search, model, &1, &CursorDirection::Left);
            }
            None
        }
        BufferMessage::Modification(count, modification) => {
            let buffer_changes = bufferline::update(mode, search, model, count, modification);
            if let Some(changes) = buffer_changes {
                model.undo.add(mode, changes);
            }
            None
        }
        BufferMessage::MoveCursor(count, direction) => {
            cursor::update_by_direction(mode, search, model, count, direction);
            None
        }
        BufferMessage::MoveViewPort(direction) => {
            viewport::update_by_direction(model, direction);
            None
        }
        BufferMessage::SaveBuffer(_) => {
            let changes = model.undo.save();
            Some(BufferResult::Changes(changes))
        }
    };

    viewport::update_by_cursor(model);

    result
}

pub fn focus_buffer(buffer: &mut Buffer) {
    if let Some(cursor) = &mut buffer.cursor {
        cursor.hide_cursor = false;
    }
}

pub fn reset_view(model: &mut Buffer) {
    let view_port = &mut model.view_port;
    view_port.horizontal_index = 0;
    view_port.vertical_index = 0;

    if let Some(cursor) = &mut model.cursor {
        cursor.vertical_index = 0;

        cursor.horizontal_index = match &cursor.horizontal_index {
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
