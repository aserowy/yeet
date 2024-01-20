use ratatui::{
    prelude::Rect,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use teywi_keymap::action::Mode;

use crate::model::buffer::Buffer;

mod style;
mod viewport;

pub fn view(mode: &Mode, model: &Buffer, frame: &mut Frame, rect: Rect) {
    let viewport_lines = viewport::get_lines(model);

    let mut lines = Vec::new();
    for (i, line) in viewport_lines.iter().enumerate() {
        let i_corrected = i + model.view_port.vertical_index;
        lines.push(update_line(i_corrected, line, mode, model));
    }

    frame.render_widget(Paragraph::new(lines), rect);
}

fn update_line<'a>(index: usize, line: &'a str, mode: &Mode, model: &'a Buffer) -> Line<'a> {
    let line_length = line.chars().count();
    let style_expansion = style::get_span_styles(index, line_length, mode, model);
    let mut spans = Vec::new();

    for (start, end, style) in style_expansion {
        if end > line_length {
            let filler_count = end - line_length;
            let mut filler = String::with_capacity(filler_count);
            filler.push_str(&" ".repeat(filler_count));

            spans.push(Span::styled(&line[start..line_length], style));
            spans.push(Span::styled(filler, style));
        } else {
            spans.push(Span::styled(&line[start..end], style));
        }
    }

    Line::from(spans)
}
