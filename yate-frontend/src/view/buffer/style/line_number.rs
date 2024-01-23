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
                (0, PositionType::LineNumberAbsolute),
                (width, PositionType::LineNumberAbsolute),
            ]
        } else {
            let position_type = match view_port.line_number {
                LineNumber::Absolute => PositionType::LineNumberAbsolute,
                LineNumber::None => unreachable!(),
                LineNumber::Relative => PositionType::LineNumberRelative,
            };

            vec![(0, position_type.clone()), (width, position_type)]
        }
    } else {
        vec![
            (0, PositionType::LineNumberAbsolute),
            (width, PositionType::LineNumberAbsolute),
        ]
    }
}
