use yeet_keymap::message::ViewPortDirection;

use crate::model::buffer::{Buffer, CursorPosition};

pub fn update_by_cursor(model: &mut Buffer) {
    if model.lines.is_empty() {
        return;
    }

    if let Some(cursor) = &model.cursor {
        let viewport = &mut model.view_port;

        if viewport.vertical_index > cursor.vertical_index {
            viewport.vertical_index = cursor.vertical_index;
        } else if viewport.vertical_index + (viewport.height - 1) < cursor.vertical_index {
            viewport.vertical_index = cursor.vertical_index - (viewport.height - 1);
        }

        let cursor_index = match cursor.horizontial_index {
            CursorPosition::Absolute {
                current,
                expanded: _,
            } => current,
            CursorPosition::End => model.lines[cursor.vertical_index].len() - 1,
            CursorPosition::None => return,
        };

        let line = &model.lines[cursor.vertical_index];
        if viewport.horizontal_index > cursor_index {
            viewport.horizontal_index = cursor_index;
        } else if viewport.horizontal_index + viewport.get_content_width(line) < cursor_index {
            viewport.horizontal_index = cursor_index - viewport.get_content_width(line)
        }
    }
}

pub fn update_by_direction(model: &mut Buffer, direction: &ViewPortDirection) {
    if model.lines.is_empty() {
        return;
    }

    match direction {
        ViewPortDirection::BottomOnCursor => {
            if let Some(cursor) = &model.cursor {
                if cursor.vertical_index < model.view_port.height {
                    model.view_port.vertical_index = 0;
                } else {
                    let index = cursor.vertical_index - model.view_port.height + 1;
                    model.view_port.vertical_index = index;
                }
            }
        }
        ViewPortDirection::CenterOnCursor => {
            if let Some(cursor) = &model.cursor {
                let index_offset = model.view_port.height / 2;
                if cursor.vertical_index < index_offset {
                    model.view_port.vertical_index = 0;
                } else {
                    model.view_port.vertical_index = cursor.vertical_index - index_offset;
                }
            }
        }
        ViewPortDirection::HalfPageDown => {
            let index_offset = model.view_port.height / 2;
            let viewport_end_index = model.view_port.vertical_index + (model.view_port.height - 1);
            let viewport_end_after_move_index = viewport_end_index + index_offset;

            if viewport_end_after_move_index < model.lines.len() {
                model.view_port.vertical_index += index_offset;
            } else if model.view_port.height > model.lines.len() {
                model.view_port.vertical_index = 0;
            } else {
                model.view_port.vertical_index = model.lines.len() - model.view_port.height;
            }

            if let Some(cursor) = &mut model.cursor {
                if cursor.vertical_index + index_offset >= model.lines.len() {
                    cursor.vertical_index = model.lines.len() - 1;
                } else {
                    cursor.vertical_index += index_offset;
                }
            }
        }
        ViewPortDirection::HalfPageUp => {
            let index_offset = model.view_port.height / 2;
            if model.view_port.vertical_index < index_offset {
                model.view_port.vertical_index = 0;
            } else {
                model.view_port.vertical_index -= index_offset;
            }

            if let Some(cursor) = &mut model.cursor {
                if cursor.vertical_index < index_offset {
                    cursor.vertical_index = 0;
                } else {
                    cursor.vertical_index -= index_offset;
                }
            }
        }
        ViewPortDirection::TopOnCursor => {
            if let Some(cursor) = &model.cursor {
                model.view_port.vertical_index = cursor.vertical_index;
            }
        }
    }
}
