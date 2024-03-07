use ratatui::{
    prelude::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use yeet_keymap::message::Mode;

use crate::model::buffer::{viewport::ViewPort, Buffer, BufferLine, Cursor, StylePartialSpan};

mod line;
mod prefix;
mod style;

pub fn view(mode: &Mode, model: &Buffer, frame: &mut Frame, rect: Rect) {
    let rendered = get_rendered_lines(model);
    let styled = get_styled_lines(&model.view_port, mode, &model.cursor, rendered);

    let rect = if model.show_border {
        let block = Block::default()
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(Color::Black));

        let inner = block.inner(rect);

        frame.render_widget(block, rect);

        inner
    } else {
        rect
    };

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
        content.push_str(&prefix::get_signs(vp, bl));
        content.push_str(&prefix::get_line_number(vp, corrected_index, cursor));
        content.push_str(&prefix::get_custom_prefix(bl));
        content.push_str(&prefix::get_border(vp));

        // NOTE: higher order (higher index) styles take precedence
        spans.extend(line::get_cursor_style_partials(vp, mode, cursor, &i, bl));
        spans.extend(correct_index(&content.chars().count(), &bl.style));

        if let Some(search) = &bl.search {
            spans.extend(correct_index(&content.chars().count(), search));
        }

        content.push_str(&bl.content);

        result.push(style::get_line(vp, content, spans));
    }

    result
}

fn correct_index(offset: &usize, style_partials: &Vec<StylePartialSpan>) -> Vec<StylePartialSpan> {
    let mut corrected_style_partials = Vec::new();

    for partial in style_partials {
        let start = partial.start + offset;
        let end = partial.end + offset;

        corrected_style_partials.push(StylePartialSpan {
            start,
            end,
            style: partial.style.clone(),
        });
    }
    corrected_style_partials
}
