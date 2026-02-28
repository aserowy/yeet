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

// TODO: try to get rid of as_deref_mut() by passing viewport as &mut Option<&mut ViewPort> and re-borrowing it when needed. It is currently needed for passing viewport to multiple messages in a row without having to clone it.
#[allow(clippy::needless_option_as_deref)] // as_deref_mut() is needed for re-borrowing
fn update_buffer(
    mut viewport: Option<&mut ViewPort>,
    mode: &Mode,
    buffer: &mut TextBuffer,
    message: &BufferMessage,
) -> Vec<BufferResult> {
    tracing::debug!("handling buffer message: {:?}", message);

    let result = match message {
        // TODO: repeat actions by count when switching from insert to normal
        // count is entered before going into insert. ChangeMode with count? Or Insert with count?
        BufferMessage::AddLine(line, sort) => {
            let current_selection = viewport.as_deref().and_then(|vp| {
                buffer
                    .lines
                    .get(vp.cursor.vertical_index)
                    .map(|line| line.content.to_stripped_string())
            });

            buffer.lines.push(line.clone());
            update_buffer(
                viewport.as_deref_mut(),
                mode,
                buffer,
                &BufferMessage::SortContent(*sort),
            );

            if let Some(current_selection) = current_selection {
                update_buffer(
                    viewport.as_deref_mut(),
                    mode,
                    buffer,
                    &BufferMessage::SetCursorToLineContent(current_selection),
                );
            }
            Vec::new()
        }
        BufferMessage::ChangeMode(from, to) => {
            if from == &Mode::Insert && to != &Mode::Insert {
                buffer.undo.close_transaction();

                if let Some(viewport) = viewport.as_deref_mut() {
                    cursor::update_by_direction(
                        mode,
                        &mut viewport.cursor,
                        buffer,
                        &1,
                        &CursorDirection::Left,
                    );
                    viewport::update_by_cursor(viewport, buffer);
                }
            }
            Vec::new()
        }
        BufferMessage::Modification(count, modification) => {
            if let Some(viewport) = viewport.as_deref_mut() {
                let changes =
                    modification::update(mode, &mut viewport.cursor, buffer, count, modification);
                viewport::update_by_cursor(viewport, buffer);

                if let Some(changes) = changes {
                    buffer.undo.add(mode, changes);
                }
            }
            Vec::new()
        }
        BufferMessage::MoveCursor(count, direction) => {
            if let Some(viewport) = viewport.as_deref_mut() {
                let result = cursor::update_by_direction(
                    mode,
                    &mut viewport.cursor,
                    buffer,
                    count,
                    direction,
                );
                viewport::update_by_cursor(viewport, buffer);

                result
            } else {
                Vec::new()
            }
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
            if let Some(viewport) = viewport.as_deref_mut() {
                if *index < viewport.cursor.vertical_index {
                    viewport.cursor.vertical_index =
                        viewport.cursor.vertical_index.saturating_sub(1);
                }
            }

            buffer.lines.remove(*index);

            if let Some(viewport) = viewport.as_deref_mut() {
                cursor::set_to_inbound_position(&mut viewport.cursor, buffer, mode);
                viewport::update_by_cursor(viewport, buffer);
            }

            Vec::new()
        }
        BufferMessage::ResetCursor => {
            if let Some(viewport) = viewport.as_deref_mut() {
                viewport.cursor.vertical_index = 0;

                viewport.cursor.horizontal_index = match &viewport.cursor.horizontal_index {
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
            buffer.lines = content.to_vec();

            if let Some(viewport) = viewport.as_deref_mut() {
                cursor::set_to_inbound_position(&mut viewport.cursor, buffer, mode);
                viewport::update_by_cursor(viewport, buffer);
            }

            Vec::new()
        }
        BufferMessage::SetCursorToLineContent(content) => {
            if let Some(viewport) = viewport.as_deref_mut() {
                let line = buffer
                    .lines
                    .iter()
                    .enumerate()
                    .find(|(_, line)| &line.content.to_stripped_string() == content);

                if let Some((index, _)) = line {
                    viewport.cursor.vertical_index = index;
                    viewport.hide_cursor_line = false;

                    cursor::set_to_inbound_position(&mut viewport.cursor, buffer, mode);
                    viewport::update_by_cursor(viewport, buffer);

                    vec![BufferResult::CursorPositionChanged]
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        }
        BufferMessage::SortContent(sort) => {
            // TODO: cursor should stay on current selection
            buffer.lines.sort_unstable_by(sort);
            if let Some(viewport) = viewport.as_deref_mut() {
                cursor::set_to_inbound_position(&mut viewport.cursor, buffer, mode);
                viewport::update_by_cursor(viewport, buffer);
            }
            Vec::new()
        }
        BufferMessage::UpdateViewPortByCursor => Vec::new(),
    };

    result
}
