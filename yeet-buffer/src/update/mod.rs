use crate::{
    message::{BufferMessage, CursorDirection},
    model::{viewport::ViewPort, BufferResult, CursorPosition, Mode, TextBuffer},
};

mod cursor;
mod find;
mod modification;
pub mod viewport;
mod word;

pub fn update(
    viewport: Option<&mut ViewPort>,
    mode: &Mode,
    buffer: &mut TextBuffer,
    messages: &[BufferMessage],
) -> Vec<BufferResult> {
    let mut actions = Vec::new();

    let mut viewport = viewport;
    for message in messages.iter() {
        actions.extend(update_buffer(
            viewport.as_deref_mut(),
            mode,
            buffer,
            message,
        ));
    }

    actions
}

fn update_buffer(
    viewport: Option<&mut ViewPort>,
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

                cursor::update_by_direction(mode, buffer, &1, &CursorDirection::Left);
                if let Some(viewport) = viewport {
                    viewport::update_by_cursor(viewport, buffer);
                }
            }
            Vec::new()
        }
        BufferMessage::Modification(count, modification) => {
            let changes = modification::update(mode, buffer, count, modification);
            if let Some(viewport) = viewport {
                viewport::update_by_cursor(viewport, buffer);
            }

            if let Some(changes) = changes {
                buffer.undo.add(mode, changes);
            }
            Vec::new()
        }
        BufferMessage::MoveCursor(count, direction) => {
            let result = cursor::update_by_direction(mode, buffer, count, direction);
            if let Some(viewport) = viewport {
                viewport::update_by_cursor(viewport, buffer);
            }

            result
            // TODO: history::add_history_entry(&mut model.history, selected.as_path());
        }
        BufferMessage::MoveViewPort(direction) => {
            let viewport = match viewport {
                Some(it) => it,
                None => return Vec::new(),
            };

            viewport::update_by_direction(viewport, buffer, direction);
            viewport::update_by_cursor(viewport, buffer);

            Vec::new()
        }
        BufferMessage::RemoveLine(index) => {
            buffer.lines.remove(*index);

            cursor::set_to_inbound_position(buffer, mode);
            if let Some(viewport) = viewport {
                viewport::update_by_cursor(viewport, buffer);
            }

            Vec::new()
        }
        BufferMessage::ResetCursor => {
            buffer.cursor.vertical_index = 0;

            buffer.cursor.horizontal_index = match &buffer.cursor.horizontal_index {
                CursorPosition::Absolute {
                    current: _,
                    expanded: _,
                } => CursorPosition::Absolute {
                    current: 0,
                    expanded: 0,
                },
                CursorPosition::End => CursorPosition::End,
                CursorPosition::None => CursorPosition::None,
            };
            if let Some(viewport) = viewport {
                viewport.horizontal_index = 0;
                viewport.vertical_index = 0;

                viewport::update_by_cursor(viewport, buffer);
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

            cursor::set_to_inbound_position(buffer, mode);
            if let Some(viewport) = viewport {
                viewport::update_by_cursor(viewport, buffer);
            }

            Vec::new()
        }
        BufferMessage::SetCursorToLineContent(content) => {
            let line = buffer
                .lines
                .iter()
                .enumerate()
                .find(|(_, line)| &line.content.to_stripped_string() == content);

            if let Some((index, _)) = line {
                buffer.cursor.vertical_index = index;
                buffer.cursor.hide_cursor_line = false;

                cursor::set_to_inbound_position(buffer, mode);
                if let Some(viewport) = viewport {
                    viewport::update_by_cursor(viewport, buffer);
                }

                vec![BufferResult::CursorPositionChanged]
            } else {
                Vec::new()
            }
        }
        BufferMessage::SortContent(sort) => {
            // TODO: cursor should stay on current selection
            buffer.lines.sort_unstable_by(sort);
            cursor::set_to_inbound_position(buffer, mode);
            if let Some(viewport) = viewport {
                viewport::update_by_cursor(viewport, buffer);
            }
            Vec::new()
        }
        BufferMessage::UpdateViewPortByCursor => Vec::new(),
    };

    result
}
