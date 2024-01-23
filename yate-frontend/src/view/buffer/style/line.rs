use crate::model::buffer::{Cursor, CursorPosition, ViewPort};

use super::{PositionType, StylePositionByLineIndex};

pub fn get_cursor_style_span(
    view_port: &ViewPort,
    cursor: &Option<Cursor>,
    lines: &[String],
) -> Option<StylePositionByLineIndex> {
    if let Some(cursor) = cursor {
        let mut cursor_positions = vec![
            (0, PositionType::CursorLine),
            (view_port.width, PositionType::CursorLine),
        ];

        let line_index = cursor.vertical_index - view_port.vertical_index;
        let line = &lines[line_index][view_port.horizontal_index..];

        let line_length = if line.chars().count() > view_port.width {
            view_port.width
        } else {
            let length = line.chars().count();
            if length == 0 {
                1
            } else {
                length
            }
        };

        let cursor_index = match &cursor.horizontial_index {
            CursorPosition::Absolute(i) => {
                let corrected_index = *i - view_port.horizontal_index;
                if corrected_index < line_length {
                    corrected_index
                } else {
                    line_length - 1
                }
            }
            CursorPosition::End => line_length - view_port.horizontal_index - 1,
            CursorPosition::None => return Some((cursor.vertical_index, cursor_positions)),
        };

        cursor_positions.extend(vec![
            (cursor_index, PositionType::Cursor),
            (cursor_index + 1, PositionType::Cursor),
        ]);

        Some((cursor.vertical_index, cursor_positions))
    } else {
        None
    }
}
