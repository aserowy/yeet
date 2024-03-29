use crate::{
    message::{BufferMessage, CursorDirection},
    model::{Buffer, BufferResult, CursorPosition, Mode, SearchModel},
};

mod cursor;
mod modification;
mod viewport;

pub fn update(
    mode: &Mode,
    search: &Option<SearchModel>,
    model: &mut Buffer,
    message: &BufferMessage,
) -> Option<BufferResult> {
    tracing::debug!("handling buffer message: {:?}", message);

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
            let buffer_changes = modification::update(mode, search, model, count, modification);
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
        BufferMessage::RemoveLine(index) => {
            model.lines.remove(*index);
            cursor::validate(mode, model);
            None
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

            None
        }
        BufferMessage::SaveBuffer => {
            let changes = model.undo.save();
            Some(BufferResult::Changes(changes))
        }
        BufferMessage::SetContent(content) => {
            // TODO: optional selection?
            model.lines = content.to_vec();
            cursor::validate(mode, model);
            None
        }
        BufferMessage::SetCursorToLineContent(content) => {
            let cursor = match &mut model.cursor {
                Some(it) => it,
                None => return None,
            };

            let line = model
                .lines
                .iter()
                .enumerate()
                .find(|(_, line)| &line.content == content);

            if let Some((index, _)) = line {
                cursor.vertical_index = index;

                cursor::validate(mode, model);
                viewport::update_by_cursor(model);

                Some(BufferResult::CursorPositionChanged)
            } else {
                None
            }
        }
        BufferMessage::SortContent(sort) => {
            // TODO: cursor should stay on current selection
            model.lines.sort_unstable_by(sort);
            cursor::validate(mode, model);
            None
        }
    };

    viewport::update_by_cursor(model);

    result
}

pub fn focus(buffer: &mut Buffer) {
    if let Some(cursor) = &mut buffer.cursor {
        cursor.hide_cursor = false;
    }
}

pub fn unfocus(buffer: &mut Buffer) {
    if let Some(cursor) = &mut buffer.cursor {
        cursor.hide_cursor = true;
    }
}
