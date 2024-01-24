use ratatui::{
    prelude::Rect,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use yate_keymap::message::Mode;

use crate::model::buffer::{Buffer, BufferLine, Cursor, ForgroundStyleSpan, ViewPort};

use self::style::{
    cursor, line_number,
    position::{self, PositionType, StylePosition},
};

mod prefix;
mod style;

pub fn view(mode: &Mode, model: &Buffer, frame: &mut Frame, rect: Rect) {
    let rendered = get_rendered_lines(model);
    let styled = get_styled_lines(&model.view_port, mode, &model.cursor, rendered);

    frame.render_widget(Paragraph::new(styled), rect);
}

pub fn get_rendered_lines(model: &Buffer) -> Vec<BufferLine> {
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
    lines: Vec<BufferLine>,
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
    for (index, bl) in lines.iter().enumerate() {
        let corrected_index = index + view_port.vertical_index;
        let mut positions: Vec<_> = positions_by_index
            .iter()
            .filter(|(i, _)| i == &corrected_index)
            .flat_map(|(_, positions)| positions.clone())
            .collect();

        // NOTE: add line specific styles here to positions with extend
        positions.extend(default_positions.clone());
        positions.extend(line_number::get_style_position(view_port, index, cursor));

        let line = if view_port.get_offset_width() > 0 {
            // NOTE: add line expansions here
            format!(
                "{} {}",
                prefix::get_line_number(view_port, corrected_index, cursor),
                bl.content
            )
        } else {
            bl.content.to_string()
        };

        result.push(Line::from(get_styled_line(
            view_port, mode, line, positions, &bl.style,
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
    let empty = vec![BufferLine {
        content: "".to_string(),
        style: Vec::new(),
    }];

    let cursor_styles = cursor::get_cursor_style_positions(view_port, cursor, &empty);

    let spans = if let Some(mut positions) = cursor_styles {
        positions.1.extend(default_positions);

        let line_number_style = line_number::get_style_position(view_port, 0, cursor);
        positions.1.extend(line_number_style);

        let mut line = prefix::get_line_number(view_port, 0, cursor);
        line.push(' ');

        get_styled_line(view_port, mode, line, positions.1, &Vec::new())
    } else {
        let line = "".to_string();

        get_styled_line(view_port, mode, line, default_positions, &Vec::new())
    };

    vec![Line::from(spans)]
}

fn get_styled_line<'a>(
    view_port: &ViewPort,
    mode: &Mode,
    line: String,
    positions: Vec<StylePosition>,
    line_styles: &[ForgroundStyleSpan],
) -> Vec<Span<'a>> {
    let sorted_positions = position::get_sorted_positions(positions);
    let span_styles = position::convert_sorted_positions_to_span_styles(mode, sorted_positions);
    let styles = style::merge_styles(view_port, span_styles, line_styles);

    style::get_spans(view_port, line, styles)
}
