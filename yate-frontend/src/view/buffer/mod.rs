use ratatui::{
    prelude::Rect,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use yate_keymap::message::Mode;

use crate::model::buffer::{Buffer, Cursor, ViewPort};

use self::style::{cursor, line_number, PositionType, StylePosition};

mod style;

pub fn view(mode: &Mode, model: &Buffer, frame: &mut Frame, rect: Rect) {
    let rendered = get_rendered_lines(model);
    let styled = get_styled_lines(&model.view_port, mode, &model.cursor, rendered);

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
    lines: Vec<String>,
) -> Vec<Line<'a>> {
    let width = view_port.get_offset_width() + view_port.content_width;
    let default_positions = vec![(0, PositionType::Default), (width, PositionType::Default)];

    if lines.is_empty() {
        return get_empty_buffer_lines(view_port, mode, cursor, default_positions);
    }

    // NOTE: add buffer styles like selection here
    let positions_by_index: Vec<_> = vec![cursor::get_cursor_style_positions(
        view_port, cursor, &lines,
    )]
    .into_iter()
    .flatten()
    .collect();

    let mut result = Vec::new();
    for (index, content) in lines.iter().enumerate() {
        let corrected_index = index + view_port.vertical_index;
        let mut positions: Vec<_> = positions_by_index
            .iter()
            .filter(|(i, _)| i == &corrected_index)
            .flat_map(|(_, positions)| positions.clone())
            .collect();

        // NOTE: add line specific styles here to positions with extend
        positions.extend(default_positions.clone());
        positions.extend(line_number::get_style_position(view_port, index, cursor));

        // NOTE: add line expansions here
        let line = format!("{:3} {}", index, content);

        result.push(Line::from(get_styled_line(
            view_port, mode, line, positions,
        )));
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
    let cursor_styles = cursor::get_cursor_style_positions(view_port, cursor, &empty);

    let spans = if let Some(mut positions) = cursor_styles {
        positions.1.extend(default_positions);

        get_styled_line(view_port, mode, "".to_string(), positions.1)
    } else {
        get_styled_line(view_port, mode, "".to_string(), default_positions)
    };

    vec![Line::from(spans)]
}

fn get_styled_line<'a>(
    view_port: &ViewPort,
    mode: &Mode,
    line: String,
    positions: Vec<StylePosition>,
) -> Vec<Span<'a>> {
    let sorted_positions = style::get_sorted_positions(positions);
    let span_to_styles = style::convert_sorted_positions_to_span_styles(mode, sorted_positions);

    style::get_spans(view_port, line, span_to_styles)
}
