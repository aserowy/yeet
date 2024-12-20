use crate::{
    message::{BufferMessage, CursorDirection},
    model::{viewport::ViewPort, BufferResult, Cursor, CursorPosition, Mode, TextBuffer},
    update::cursor::{set_outbound_cursor_to_inbound_position, update_cursor_by_direction},
};

mod cursor;
mod find;
mod modification;
mod viewport;
mod word;

pub fn update_buffer(
    viewport: &mut ViewPort,
    cursor: &mut Option<Cursor>,
    mode: &Mode,
    buffer: &mut TextBuffer,
    message: &BufferMessage,
) -> Vec<BufferResult> {
    tracing::debug!("handling buffer message: {:?}", message);

    let result = match message {
        // TODO: repeat actions by count when switching from insert to normal
        // count is entered before going into insert. ChangeMode with count? Or Insert with count?
        BufferMessage::ChangeMode(from, to) => {
            if from == &Mode::Insert && to != &Mode::Insert {
                buffer.undo.close_transaction();

                if let Some(cursor) = cursor {
                    update_cursor_by_direction(cursor, mode, buffer, &1, &CursorDirection::Left);
                }
            }
            Vec::new()
        }
        BufferMessage::Modification(count, modification) => {
            if let Some(cursor) = cursor {
                let changes = modification::update(cursor, mode, buffer, count, modification);
                if let Some(changes) = changes {
                    buffer.undo.add(mode, changes);
                }
            }
            Vec::new()
        }
        BufferMessage::MoveCursor(count, direction) => {
            if let Some(cursor) = cursor {
                update_cursor_by_direction(cursor, mode, buffer, count, direction)
            } else {
                Vec::new()
            }
            // TODO: history::add_history_entry(&mut model.history, selected.as_path());
        }
        BufferMessage::MoveViewPort(direction) => {
            viewport::update_by_direction(viewport, cursor, buffer, direction);
            Vec::new()
        }
        BufferMessage::RemoveLine(index) => {
            buffer.lines.remove(*index);

            if let Some(cursor) = cursor {
                set_outbound_cursor_to_inbound_position(cursor, mode, buffer);
            }

            Vec::new()
        }
        BufferMessage::ResetCursor => {
            viewport.horizontal_index = 0;
            viewport.vertical_index = 0;

            if let Some(cursor) = cursor {
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

            if let Some(cursor) = cursor {
                set_outbound_cursor_to_inbound_position(cursor, mode, buffer);
            }

            Vec::new()
        }
        BufferMessage::SetCursorToLineContent(content) => {
            let cursor = match cursor {
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

                set_outbound_cursor_to_inbound_position(cursor, mode, buffer);
                viewport::update_by_cursor(viewport, cursor, buffer);

                vec![BufferResult::CursorPositionChanged]
            } else {
                Vec::new()
            }
        }
        BufferMessage::SortContent(sort) => {
            // TODO: cursor should stay on current selection
            buffer.lines.sort_unstable_by(sort);
            if let Some(cursor) = cursor {
                set_outbound_cursor_to_inbound_position(cursor, mode, buffer);
            }
            Vec::new()
        }
        BufferMessage::UpdateViewPortByCursor => Vec::new(),
    };

    if let Some(cursor) = cursor {
        viewport::update_by_cursor(viewport, cursor, buffer);
    }

    result
}

pub fn focus_buffer(cursor: &mut Option<Cursor>) {
    if let Some(cursor) = cursor {
        cursor.hide_cursor = false;
    }
}

pub fn unfocus_buffer(cursor: &mut Option<Cursor>) {
    if let Some(cursor) = cursor {
        cursor.hide_cursor = true;
    }
}
