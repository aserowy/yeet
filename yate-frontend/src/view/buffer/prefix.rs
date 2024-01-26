use crate::{
    model::buffer::{BufferLine, Cursor, LineNumber, StylePartialSpan, ViewPort},
    view::buffer::style::{LINE_NUMBER_ABS_STYLE_PARTIAL, LINE_NUMBER_REL_STYLE_PARTIAL},
};

pub fn get_border(vp: &ViewPort) -> String {
    " ".repeat(vp.get_border_width()).to_string()
}

pub fn get_custom_prefix(line: &BufferLine) -> String {
    if let Some(prefix) = &line.prefix {
        prefix.to_owned()
    } else {
        "".to_string()
    }
}

pub fn get_line_number(vp: &ViewPort, index: usize, cursor: &Option<Cursor>) -> String {
    if vp.line_number == LineNumber::None {
        return "".to_string();
    }

    let width = vp.get_line_number_width();
    let number = {
        let number_string = (index + 1).to_string();
        if let Some(index) = number_string.char_indices().nth_back(width - 1) {
            number_string[index.0..].to_string()
        } else {
            number_string
        }
    };

    if let Some(cursor) = cursor {
        if cursor.vertical_index == index {
            return format!("{:<width$}", number);
        }
    }

    match vp.line_number {
        LineNumber::_Absolute => format!("{:>width$} ", number),
        LineNumber::None => "".to_string(),
        LineNumber::Relative => {
            if let Some(cursor) = cursor {
                let relative = if cursor.vertical_index > index {
                    cursor.vertical_index - index
                } else {
                    index - cursor.vertical_index
                };

                format!("{:>width$}", relative)
            } else {
                format!("{:>width$}", number)
            }
        }
    }
}

pub fn get_line_number_style_partials(
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
            vec![(0, width, LINE_NUMBER_ABS_STYLE_PARTIAL.clone())]
        } else {
            let style_partial = match view_port.line_number {
                LineNumber::_Absolute => LINE_NUMBER_ABS_STYLE_PARTIAL.clone(),
                LineNumber::None => unreachable!(),
                LineNumber::Relative => LINE_NUMBER_REL_STYLE_PARTIAL.clone(),
            };

            vec![(0, width, style_partial)]
        }
    } else {
        vec![(0, width, LINE_NUMBER_ABS_STYLE_PARTIAL.clone())]
    }
}
