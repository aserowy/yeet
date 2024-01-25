use ratatui::{prelude::Rect, text::Line, widgets::Paragraph, Frame};
use yate_keymap::message::Mode;

use crate::model::buffer::{Buffer, BufferLine, Cursor, StylePartialSpan, ViewPort};

use self::style::{cursor, line_number};

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
    if lines.is_empty() {
        return get_empty_buffer_lines(vp, mode, cursor);
    }

    let offset = vp.get_offset_width();
    let mut result = Vec::new();
    for (index, bl) in lines.iter().enumerate() {
        let corrected_index = index + vp.vertical_index;

        let mut style_partials: Vec<_> = Vec::new();
        // NOTE: higher order (higher index) styles take precedence
        style_partials.extend(line_number::get_style_partials(vp, cursor, &index));
        style_partials.extend(cursor::get_style_partials(vp, mode, cursor, &index, bl));
        style_partials.extend(correct_index(&offset, &bl.style));

        let line = if offset > 0 {
            let mut content = String::new();
            content.push_str(&prefix::get_line_number(vp, corrected_index, cursor));
            content.push(' ');
            content.push_str(&bl.content);

            content
        } else {
            bl.content.to_string()
        };

        result.push(style::get_line(vp, line, style_partials));
    }

    result
}

fn get_empty_buffer_lines<'a>(
    view_port: &ViewPort,
    mode: &Mode,
    cursor: &Option<Cursor>,
) -> Vec<Line<'a>> {
    let empty = BufferLine {
        content: "".to_string(),
        style: Vec::new(),
    };

    let mut cursor_styles = cursor::get_style_partials(view_port, mode, cursor, &0, &empty);
    let spans = if cursor_styles.is_empty() {
        style::get_line(view_port, "".to_string(), Vec::new())
    } else {
        let line_number_style = line_number::get_style_partials(view_port, cursor, &0);
        cursor_styles.extend(line_number_style);

        let mut line = prefix::get_line_number(view_port, 0, cursor);
        line.push(' ');

        style::get_line(view_port, line, cursor_styles)
    };

    vec![spans]
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
