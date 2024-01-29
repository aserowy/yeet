use std::path::PathBuf;

use crate::model::{
    self,
    buffer::{Buffer, Cursor, CursorPosition},
    history::History,
};

use super::buffer::viewport;

pub fn set_cursor_index(path: &PathBuf, history: &History, buffer: &mut Buffer) -> bool {
    if let Some(history) = model::history::get_selection(history, path) {
        let line = buffer
            .lines
            .iter()
            .enumerate()
            .find(|(_, line)| line.content == history);

        if let Some((index, _)) = line {
            if let Some(cursor) = &mut buffer.cursor {
                cursor.vertical_index = index;
            } else {
                buffer.cursor = Some(Cursor {
                    horizontial_index: CursorPosition::None,
                    vertical_index: index,
                    ..Default::default()
                });
            }

            viewport::update_by_cursor(buffer);

            return true;
        }
    }

    false
}
