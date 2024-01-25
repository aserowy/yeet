use crate::model::buffer::{Cursor, LineNumber, ViewPort};

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

pub fn get_border(vp: &ViewPort) -> String {
    " ".repeat(vp.get_border_width()).to_string()
}
