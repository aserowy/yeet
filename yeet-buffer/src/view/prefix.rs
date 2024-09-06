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

// pub fn get_line_number_style_partials(
//     vp: &ViewPort,
//     cursor: &Option<Cursor>,
//     index: &usize,
// ) -> Vec<StylePartialSpan> {
//     let start = vp.sign_column_width;
//
//     let end = start + vp.get_line_number_width();
//     if start == end {
//         return Vec::new();
//     }
//
//     if let Some(cursor) = cursor {
//         if cursor.vertical_index - vp.vertical_index == *index {
//             vec![StylePartialSpan {
//                 start,
//                 end,
//                 style: LINE_NUMBER_ABS_STYLE_PARTIAL.clone(),
//             }]
//         } else {
//             let style_partial = match vp.line_number {
//                 LineNumber::_Absolute => LINE_NUMBER_ABS_STYLE_PARTIAL.clone(),
//                 LineNumber::None => unreachable!(),
//                 LineNumber::Relative => LINE_NUMBER_REL_STYLE_PARTIAL.clone(),
//             };
//
//             vec![StylePartialSpan {
//                 start,
//                 end,
//                 style: style_partial,
//             }]
//         }
//     } else {
//         vec![StylePartialSpan {
//             start,
//             end,
//             style: LINE_NUMBER_ABS_STYLE_PARTIAL.clone(),
//         }]
//     }
// }

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
        .map(|s| s.content.clone())
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
