use crate::{
    message::{BufferMessage, CursorDirection},
    model::{viewport::ViewPort, Buffer, BufferResult, CursorPosition, Mode},
    update::cursor::{update_cursor_by_direction, validate_cursor_position},
};

mod cursor;
mod find;
mod modification;
mod viewport;
mod word;

pub fn update_buffer(
    viewport: &mut ViewPort,
    mode: &Mode,
    buffer: &mut Buffer,
    message: &BufferMessage,
) -> Vec<BufferResult> {
    tracing::debug!("handling buffer message: {:?}", message);

    let result = match message {
        // TODO: repeat actions by count when switching from insert to normal
        // count is entered before going into insert. ChangeMode with count? Or Insert with count?
        BufferMessage::ChangeMode(from, to) => {
            if from == &Mode::Insert && to != &Mode::Insert {
                buffer.undo.close_transaction();
                update_cursor_by_direction(mode, buffer, &1, &CursorDirection::Left);
            }
            Vec::new()
        }
        BufferMessage::Modification(count, modification) => {
            let buffer_changes = modification::update(mode, buffer, count, modification);
            if let Some(changes) = buffer_changes {
                buffer.undo.add(mode, changes);
            }
            Vec::new()
        }
        BufferMessage::MoveCursor(count, direction) => {
            update_cursor_by_direction(mode, buffer, count, direction)
            // TODO: history::add_history_entry(&mut model.history, selected.as_path());
        }
        BufferMessage::MoveViewPort(direction) => {
            viewport::update_by_direction(viewport, buffer, direction);
            Vec::new()
        }
        BufferMessage::RemoveLine(index) => {
            buffer.lines.remove(*index);
            validate_cursor_position(mode, buffer);
            Vec::new()
        }
        BufferMessage::ResetCursor => {
            viewport.horizontal_index = 0;
            viewport.vertical_index = 0;

            if let Some(cursor) = &mut buffer.cursor {
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
            let changes = buffer.undo.save();
            vec![BufferResult::Changes(changes)]
        }
        BufferMessage::SetContent(content) => {
            // TODO: optional selection?
            buffer.lines = content.to_vec();
            validate_cursor_position(mode, buffer);
            Vec::new()
        }
        BufferMessage::SetCursorToLineContent(content) => {
            let cursor = match &mut buffer.cursor {
                Some(it) => it,
                None => return Vec::new(),
            };

            let line = buffer
                .lines
                .iter()
                .enumerate()
                .find(|(_, line)| &line.content.to_stripped_string() == content);

            if let Some((index, _)) = line {
                cursor.vertical_index = index;
                cursor.hide_cursor_line = false;

                validate_cursor_position(mode, buffer);
                viewport::update_by_cursor(viewport, buffer);

                vec![BufferResult::CursorPositionChanged]
            } else {
                Vec::new()
            }
        }
        BufferMessage::SortContent(sort) => {
            // TODO: cursor should stay on current selection
            buffer.lines.sort_unstable_by(sort);
            validate_cursor_position(mode, buffer);
            Vec::new()
        }
        BufferMessage::UpdateViewPortByCursor => Vec::new(),
    };

    viewport::update_by_cursor(viewport, buffer);

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
