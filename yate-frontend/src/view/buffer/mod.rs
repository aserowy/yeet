use ratatui::{prelude::Rect, text::Line, widgets::Paragraph, Frame};
use yate_keymap::message::Mode;

use crate::model::buffer::{Buffer, Cursor, ViewPort};

use self::style::{line, PositionType, StylePosition};

mod style;

pub fn view(mode: &Mode, model: &Buffer, frame: &mut Frame, rect: Rect) {
    let rendered = get_rendered_lines(model);
    let styled = get_styled_lines(&model.view_port, mode, &model.cursor, &rendered);

    frame.render_widget(Paragraph::new(styled), rect);
}

pub fn get_rendered_lines(model: &Buffer) -> Vec<String> {
    model
        .lines
        .iter()
        .skip(model.view_port.vertical_index)
        .take(model.view_port.height)
        .map(|line| line.to_owned())
        .collect()
}

pub fn get_styled_lines<'a>(
    view_port: &ViewPort,
    mode: &Mode,
    cursor: &Option<Cursor>,
    lines: &'a Vec<String>,
) -> Vec<Line<'a>> {
    let default_positions = vec![
        (0, PositionType::Default),
        (view_port.width, PositionType::Default),
    ];

    if lines.is_empty() {
        return get_empty_buffer_lines(view_port, mode, cursor, default_positions);
    }

    // NOTE: add buffer styles like selection here
    let positions_by_index: Vec<_> = vec![line::get_cursor_style_span(view_port, cursor, lines)]
        .into_iter()
        .flatten()
        .collect();

    // TODO: offset with relative line numbers string for signs and number not
    // here! special functions for signs and numbers

    let mut result = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let corrected_index = index + view_port.vertical_index;
        let mut positions: Vec<_> = positions_by_index
            .iter()
            .filter(|(i, _)| i == &corrected_index)
            .flat_map(|(_, positions)| positions.clone())
            .collect();

        // NOTE: add line specific styles here to positions with extend
        positions.extend(default_positions.clone());

        result.push(get_styled_line(view_port, mode, line, positions));
    }

    result
}

fn get_empty_buffer_lines<'a>(
    view_port: &ViewPort,
    mode: &Mode,
    cursor: &Option<Cursor>,
    default_positions: Vec<StylePosition>,
) -> Vec<Line<'a>> {
    let empty = vec!["".to_string()];
    let cursor_styles = line::get_cursor_style_span(view_port, cursor, &empty);

    if let Some(mut positions) = cursor_styles {
        positions.1.extend(default_positions);
        return vec![get_styled_line(view_port, mode, "", positions.1)];
    }

    vec![get_styled_line(view_port, mode, "", default_positions)]
}

fn get_styled_line<'a>(
    view_port: &ViewPort,
    mode: &Mode,
    line: &'a str,
    positions: Vec<StylePosition>,
) -> Line<'a> {
    let sorted_positions = style::get_sorted_positions(positions);
    let span_to_styles = style::convert_sorted_positions_to_span_styles(mode, sorted_positions);

    Line::from(style::get_spans(view_port, line, span_to_styles))
}
