use std::cmp::Reverse;

use crate::model::{
    viewport::{LineNumber, ViewPort},
    BufferLine, Cursor,
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
            return format!("\x1b[1m{:<width$}\x1b[0m", number);
        }
    }

    match vp.line_number {
        LineNumber::Absolute => format!("{:>width$} ", number),
        LineNumber::None => "".to_string(),
        LineNumber::Relative => {
            if let Some(cursor) = cursor {
                let relative = if cursor.vertical_index > index {
                    cursor.vertical_index - index
                } else {
                    index - cursor.vertical_index
                };

                format!("\x1b[90m{:>width$}\x1b[0m", relative)
            } else {
                format!("{:>width$}", number)
            }
        }
    }
}

pub fn get_signs(vp: &ViewPort, bl: &BufferLine) -> String {
    let max_sign_count = vp.sign_column_width;

    let mut filtered: Vec<_> = bl
        .signs
        .iter()
        .filter(|s| !vp.hidden_sign_ids.contains(&s.id))
        .collect();

    filtered.sort_unstable_by_key(|s| Reverse(s.priority));

    let signs = filtered
        .iter()
        .take(max_sign_count)
        .map(|s| format!("{}{}\x1b[0m", s.style, s.content))
        .collect::<String>();

    format!("{:<max_sign_count$}", signs)
}

// pub fn get_sign_style_partials(vp: &ViewPort, bl: &BufferLine) -> Vec<StylePartialSpan> {
//     let max_sign_count = vp.sign_column_width;
//
//     let mut filtered: Vec<_> = bl
//         .signs
//         .iter()
//         .filter(|s| !vp.hidden_sign_ids.contains(&s.id))
//         .collect();
//
//     filtered.sort_unstable_by_key(|s| Reverse(s.priority));
//
//     filtered
//         .iter()
//         .take(max_sign_count)
//         .enumerate()
//         .flat_map(|(i, s)| {
//             let mut styles = Vec::new();
//             for style in &s.style {
//                 styles.push(StylePartialSpan {
//                     start: i,
//                     end: i + 1,
//                     style: style.clone(),
//                 });
//             }
//             styles
//         })
//         .collect()
// }
