use ratatui::{prelude::Rect, text::Line, widgets::Paragraph, Frame};
use yate_keymap::message::Mode;

use crate::model::buffer::{Buffer, BufferLine, Cursor, StylePartialSpan, ViewPort};

mod line;
mod prefix;
mod style;

pub fn view(mode: &Mode, model: &Buffer, frame: &mut Frame, rect: Rect) {
    let rendered = get_rendered_lines(model);
    let styled = get_styled_lines(&model.view_port, mode, &model.cursor, rendered);

    frame.render_widget(Paragraph::new(styled), rect);
}

fn get_rendered_lines(model: &Buffer) -> Vec<BufferLine> {
    model
        .lines
        .iter()
        .skip(model.view_port.vertical_index)
        .take(model.view_port.height)
        .map(|line| line.to_owned())
        .collect()
}

fn get_styled_lines<'a>(
    vp: &ViewPort,
    mode: &Mode,
    cursor: &Option<Cursor>,
    lines: Vec<BufferLine>,
) -> Vec<Line<'a>> {
    let lines = if lines.is_empty() {
        vec![BufferLine::default()]
    } else {
        lines
    };

    let mut result = Vec::new();
    for (i, bl) in lines.iter().enumerate() {
        let corrected_index = i + vp.vertical_index;

        let mut spans: Vec<_> = Vec::new();
        let mut content = String::new();
        spans.extend(prefix::get_line_number_style_partials(vp, cursor, &i));
        content.push_str(&prefix::get_line_number(vp, corrected_index, cursor));
        content.push_str(&prefix::get_border(vp));

        // NOTE: higher order (higher index) styles take precedence
        spans.extend(line::get_cursor_style_partials(vp, mode, cursor, &i, bl));
        spans.extend(correct_index(&content.chars().count(), &bl.style));
        content.push_str(&bl.content);

        result.push(style::get_line(vp, content, spans));
    }

    result
}

fn correct_index(offset: &usize, style_partials: &Vec<StylePartialSpan>) -> Vec<StylePartialSpan> {
    let mut corrected_style_partials = Vec::new();

    for (start, end, style) in style_partials {
        let s = start + offset;
        let e = end + offset;

        corrected_style_partials.push((s, e, style.clone()));
    }
    corrected_style_partials
}
