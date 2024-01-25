use crate::{
    model::buffer::{Cursor, LineNumber, StylePartialSpan, ViewPort},
    view::buffer::style::{LINE_NUMBER_ABSOLUTE_STYLE_PARTIAL, LINE_NUMBER_RELATIVE_STYLE_PARTIAL},
};

pub fn get_style_partials(
    view_port: &ViewPort,
    cursor: &Option<Cursor>,
    index: &usize,
) -> Vec<StylePartialSpan> {
    let width = view_port.get_line_number_width();
    if width == 0 {
        return Vec::new();
    }

    if let Some(cursor) = cursor {
        if cursor.vertical_index - view_port.vertical_index == *index {
            vec![(0, width, LINE_NUMBER_ABSOLUTE_STYLE_PARTIAL.clone())]
        } else {
            let style_partial = match view_port.line_number {
                LineNumber::_Absolute => LINE_NUMBER_ABSOLUTE_STYLE_PARTIAL.clone(),
                LineNumber::None => unreachable!(),
                LineNumber::Relative => LINE_NUMBER_RELATIVE_STYLE_PARTIAL.clone(),
            };

            vec![(0, width, style_partial)]
        }
    } else {
        vec![(0, width, LINE_NUMBER_ABSOLUTE_STYLE_PARTIAL.clone())]
    }
}
