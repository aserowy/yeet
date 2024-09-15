use crate::{
    message::{BufferMessage, CursorDirection},
    model::{Buffer, BufferResult, CursorPosition, Mode},
    update::cursor::{update_cursor_by_direction, validate_cursor_position},
};

mod cursor;
mod find;
mod modification;
mod viewport;
mod word;

pub fn update_buffer(
    mode: &Mode,
    model: &mut Buffer,
    message: &BufferMessage,
) -> Vec<BufferResult> {
    tracing::debug!("handling buffer message: {:?}", message);

    let result = match message {
        // TODO: repeat actions by count when switching from insert to normal
        // count is entered before going into insert. ChangeMode with count? Or Insert with count?
        BufferMessage::ChangeMode(from, to) => {
            if from == &Mode::Insert && to != &Mode::Insert {
                model.undo.close_transaction();
                update_cursor_by_direction(mode, model, &1, &CursorDirection::Left);
            }
            Vec::new()
        }
        BufferMessage::Modification(count, modification) => {
            let buffer_changes = modification::update(mode, model, count, modification);
            if let Some(changes) = buffer_changes {
                model.undo.add(mode, changes);
            }
            Vec::new()
        }
        BufferMessage::MoveCursor(count, direction) => {
            update_cursor_by_direction(mode, model, count, direction)
        }
        BufferMessage::MoveViewPort(direction) => {
            viewport::update_by_direction(model, direction);
            Vec::new()
        }
        BufferMessage::RemoveLine(index) => {
            model.lines.remove(*index);
            validate_cursor_position(mode, model);
            Vec::new()
        }
        BufferMessage::ResetCursor => {
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

            Vec::new()
        }
        BufferMessage::SaveBuffer => {
            let changes = model.undo.save();
            vec![BufferResult::Changes(changes)]
        }
        BufferMessage::SetContent(content) => {
            // TODO: optional selection?
            model.lines = content.to_vec();
            validate_cursor_position(mode, model);
            Vec::new()
        }
        BufferMessage::SetCursorToLineContent(content) => {
            let cursor = match &mut model.cursor {
                Some(it) => it,
                None => return Vec::new(),
            };

            let line = model
                .lines
                .iter()
                .enumerate()
                .find(|(_, line)| &line.content.to_stripped_string() == content);

            if let Some((index, _)) = line {
                cursor.vertical_index = index;
                cursor.hide_cursor_line = false;

                validate_cursor_position(mode, model);
                viewport::update_by_cursor(model);

                vec![BufferResult::CursorPositionChanged]
            } else {
                Vec::new()
            }
        }
        BufferMessage::SortContent(sort) => {
            // TODO: cursor should stay on current selection
            model.lines.sort_unstable_by(sort);
            validate_cursor_position(mode, model);
            Vec::new()
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

pub fn unfocus_buffer(buffer: &mut Buffer) {
    if let Some(cursor) = &mut buffer.cursor {
        cursor.hide_cursor = true;
    }
}
