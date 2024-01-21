use crate::model::buffer::{Buffer, CursorPosition};

pub fn update_by_cursor(model: &mut Buffer) {
    if let Some(cursor) = &model.cursor {
        let viewport = &mut model.view_port;

        if viewport.vertical_index > cursor.vertical_index {
            viewport.vertical_index = cursor.vertical_index;
        } else if viewport.vertical_index + (viewport.height - 1) < cursor.vertical_index {
            viewport.vertical_index = cursor.vertical_index - (viewport.height - 1);
        }

        let cursor_index = match cursor.horizontial_index {
            CursorPosition::Absolute(n) => n,
            CursorPosition::End => model.lines[cursor.vertical_index].chars().count() - 1,
        };

        if viewport.horizontal_index > cursor_index {
            viewport.horizontal_index = cursor_index;
        } else if viewport.horizontal_index + viewport.width < cursor_index {
            viewport.horizontal_index = cursor_index - viewport.width;
        }
    }
}
