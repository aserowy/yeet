use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::model::buffer::{StylePartial, StylePartialSpan, ViewPort};

type StyleSpan = (usize, usize, Style);

pub const CURSOR_INSERT_STYLE_PARTIAL: StylePartial = StylePartial::Modifier(Modifier::REVERSED);
pub const CURSOR_NORMAL_STYLE_PARTIAL: StylePartial = StylePartial::Modifier(Modifier::REVERSED);
pub const CURSORLINE_STYLE_PARTIAL: StylePartial = StylePartial::Background(Color::DarkGray);
pub const LINE_NUMBER_ABS_STYLE_PARTIAL: StylePartial = StylePartial::Foreground(Color::White);
pub const LINE_NUMBER_REL_STYLE_PARTIAL: StylePartial = StylePartial::Foreground(Color::DarkGray);

pub fn get_line<'a>(
    vp: &ViewPort,
    line: String,
    style_partials: Vec<StylePartialSpan>,
) -> Line<'a> {
    let style_spans = merge_style_partial_spans(vp, style_partials);
    let spans = get_spans(vp, line, style_spans);

    Line::from(spans)
}

fn merge_style_partial_spans(
    view_port: &ViewPort,
    style_partials: Vec<StylePartialSpan>,
) -> Vec<StyleSpan> {
    let mut result = vec![(0, view_port.width, Style::default())];

    for (sp_start, sp_end, sp_style) in &style_partials {
        let mut styles = Vec::new();
        for (start, end, style) in &result {
            if sp_start > end || sp_end < start {
                styles.push((*start, *end, *style));
                continue;
            }

            let split_start = if sp_start > start { sp_start } else { start };
            let split_end = if sp_end < end { sp_end } else { end };

            let mixed_style = match sp_style {
                StylePartial::Foreground(clr) => style.fg(*clr),
                StylePartial::Modifier(mdfr) => style.add_modifier(*mdfr),
                StylePartial::Background(clr) => style.bg(*clr),
            };

            if split_start == start && split_end == end {
                styles.push((*split_start, *split_end, mixed_style));
            } else if split_start == start {
                styles.push((*start, *split_end, mixed_style));
                styles.push((*split_end, *end, *style));
            } else if split_end == end {
                styles.push((*start, *split_start, *style));
                styles.push((*split_start, *end, mixed_style));
            } else {
                styles.push((*start, *split_start, *style));
                styles.push((*split_start, *split_end, mixed_style));
                styles.push((*split_end, *end, *style));
            }
        }

        if !styles.is_empty() {
            result = styles;
        }
    }

    result
}

fn get_spans<'a>(view_port: &ViewPort, line: String, style_spans: Vec<StyleSpan>) -> Vec<Span<'a>> {
    let line = line.chars().skip(view_port.horizontal_index);

    let line_count = line.clone().count();
    let line_length = if line_count > view_port.width {
        view_port.width
    } else {
        line_count
    };

    let mut spans = Vec::new();
    for (start, end, style) in style_spans {
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
