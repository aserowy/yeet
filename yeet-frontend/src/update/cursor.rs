use std::path::Path;

use crate::model::{
    buffer::{Buffer, Cursor, CursorPosition},
    history::History,
};

use super::buffer::viewport;

pub fn set_cursor_index(selection: &str, buffer: &mut Buffer) -> bool {
    let line = buffer
        .lines
        .iter()
        .enumerate()
        .find(|(_, line)| line.content == selection);

    if let Some((index, _)) = line {
        if let Some(cursor) = &mut buffer.cursor {
            cursor.vertical_index = index;
        } else {
            buffer.cursor = Some(Cursor {
                horizontal_index: CursorPosition::None,
                vertical_index: index,
                ..Default::default()
            });
        }

        viewport::update_by_cursor(buffer);

        true
    } else {
        false
    }
}

pub fn set_cursor_index_with_history(path: &Path, history: &History, buffer: &mut Buffer) -> bool {
    if let Some(history) = history.get_selection(path) {
        set_cursor_index(history, buffer)
    } else {
        false
    }
}
