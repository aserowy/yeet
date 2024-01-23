use crate::{
    model::buffer::{Cursor, LineNumber, ViewPort},
    view::buffer::style::PositionType,
};

use super::StylePosition;

pub fn get_style_position(
    view_port: &ViewPort,
    index: usize,
    cursor: &Option<Cursor>,
) -> Vec<StylePosition> {
    let width = view_port.get_line_number_width();
    if width == 0 {
        return Vec::new();
    }

    if let Some(cursor) = cursor {
        if cursor.vertical_index == index {
            vec![
                (0, PositionType::LineNumber(LineNumber::Absolute)),
                (width, PositionType::LineNumber(LineNumber::Absolute)),
            ]
        } else {
            let position_type = PositionType::LineNumber(view_port.line_number.clone());
            vec![(0, position_type.clone()), (width, position_type)]
        }
    } else {
        vec![
            (0, PositionType::LineNumber(LineNumber::Absolute)),
            (width, PositionType::LineNumber(LineNumber::Absolute)),
        ]
    }
}
