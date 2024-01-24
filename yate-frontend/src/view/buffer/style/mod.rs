use ratatui::{style::Style, text::Span};

use crate::model::buffer::{ForegroundStyle, ViewPort};

pub mod cursor;
pub mod line_number;
pub mod position;

type StyleSpan = (usize, usize, Style);

pub fn merge_styles(
    view_port: &ViewPort,
    span_styles: Vec<(usize, usize, Style)>,
    line_styles: &[(usize, usize, ForegroundStyle)],
) -> Vec<(usize, usize, Style)> {
    let mut styles = Vec::new();
    for (s_start, s_end, s_style) in &span_styles {
        let mut processed = false;
        for (l_start, l_end, l_style) in line_styles {
            let l_s = l_start + view_port.get_offset_width();
            let l_e = l_end + view_port.get_offset_width();

            if &l_s > s_end || &l_e < s_start {
                continue;
            }

            processed = true;

            let split_start = if &l_s > s_start { &l_s } else { s_start };
            let split_end = if &l_e < s_end { &l_e } else { s_end };

            let mixed_style = match l_style {
                ForegroundStyle::Color(clr) => s_style.fg(*clr),
                ForegroundStyle::_Modifier(mdfr) => s_style.add_modifier(*mdfr),
            };

            if split_start == s_start && split_end == s_end {
                styles.push((*split_start, *split_end, mixed_style));
            } else if split_start == s_start {
                styles.push((*s_start, *split_end, mixed_style));
                styles.push((*split_end, *s_end, *s_style));
            } else if split_end == s_end {
                styles.push((*s_start, *split_start, *s_style));
                styles.push((*split_start, *s_end, mixed_style));
            } else {
                styles.push((*s_start, *split_start, *s_style));
                styles.push((*split_start, *split_end, mixed_style));
                styles.push((*split_end, *s_end, *s_style));
            }
        }

        if !processed {
            styles.push((*s_start, *s_end, *s_style));
        }
    }

    styles
}

pub fn get_spans<'a>(
    view_port: &ViewPort,
    line: String,
    span_to_styles: Vec<StyleSpan>,
) -> Vec<Span<'a>> {
    let line = line.chars().skip(view_port.horizontal_index);

    let line_count = line.clone().count();
    let line_length = if line_count > view_port.content_width {
        view_port.content_width
    } else {
        line_count
    };

    let mut spans = Vec::new();
    for (start, end, style) in span_to_styles {
        if end > line_length {
            let filler_count = end - line_length;
            let mut filler = String::with_capacity(filler_count);
            filler.push_str(&" ".repeat(filler_count));

            if line_length > start {
                let part = line
                    .clone()
                    .skip(start)
                    .take(line_length - start)
                    .collect::<String>();

                spans.push(Span::styled(part, style));
            }

            spans.push(Span::styled(filler, style));
        } else {
            let part = line
                .clone()
                .skip(start)
                .take(end - start)
                .collect::<String>();

            spans.push(Span::styled(part, style));
        }
    }

    spans
}
