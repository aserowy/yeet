use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use yate_keymap::message::Mode;

use crate::model::buffer::{Cursor, ViewPort};

mod line;

type StyleSpan = (usize, Vec<(usize, PositionType)>);

#[derive(Clone, Debug, PartialEq)]
enum PositionType {
    Cursor,
    CursorLine,
    Default,
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

    let positions_by_index: Vec<_> = vec![line::get_cursor_style_span(view_port, cursor, &lines)]
        .into_iter()
        .flat_map(|span| span)
        .collect();

    let mut result = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let corrected_index = index + view_port.vertical_index;
        let mut positions: Vec<_> = positions_by_index
            .iter()
            .filter(|(i, _)| i == &corrected_index)
            .flat_map(|(_, positions)| positions)
            .collect();

        positions.extend(default_positions.iter());

        // NOTE: add line specific styles here to positions with extend

        let sorted_positions = get_sorted_positions(positions);
        let span_to_styles = convert_sorted_positions_to_span_styles(mode, sorted_positions);

        result.push(Line::from(get_spans(view_port, line, span_to_styles)));
    }

    result
}

fn get_sorted_positions(positions: Vec<&(usize, PositionType)>) -> Vec<(usize, Vec<PositionType>)> {
    let mut result: Vec<(usize, Vec<PositionType>)> = Vec::new();
    for (index, position_type) in positions {
        if let Some((_, position_types)) = result.iter_mut().find(|(i, _)| i == index) {
            position_types.push(position_type.clone());
        } else {
            result.push((*index, vec![position_type.clone()]));
        }
    }

    result.sort_by(|(a, _), (b, _)| a.cmp(b));

    result
}

fn convert_sorted_positions_to_span_styles(
    mode: &Mode,
    sorted_positions: Vec<(usize, Vec<PositionType>)>,
) -> Vec<(usize, usize, Style)> {
    let mut expansions = Vec::new();
    let mut last_position_index = None;
    let mut active_position_types = Vec::new();

    for (index, types) in sorted_positions {
        match last_position_index {
            Some(lpi) => {
                let style = get_style(mode, &active_position_types);

                expansions.push((lpi, index, style));
                last_position_index = Some(index);
            }
            None => last_position_index = Some(index),
        };

        for pt in types {
            if active_position_types.contains(&pt) {
                active_position_types.retain(|t| t != &pt);
            } else {
                active_position_types.push(pt);
            }
        }
    }

    expansions
}

fn get_spans<'a>(
    view_port: &ViewPort,
    line: &'a str,
    span_to_styles: Vec<(usize, usize, Style)>,
) -> Vec<Span<'a>> {
    let line = &line[view_port.horizontal_index..];

    let line_length = if line.chars().count() > view_port.width {
        view_port.width
    } else {
        line.chars().count()
    };

    let mut spans = Vec::new();
    for (start, end, style) in span_to_styles {
        if end > line_length {
            let filler_count = end - line_length;
            let mut filler = String::with_capacity(filler_count);
            filler.push_str(&" ".repeat(filler_count));

            if line_length > 0 {
                spans.push(Span::styled(&line[start..line_length], style));
            }
            spans.push(Span::styled(filler, style));
        } else {
            spans.push(Span::styled(&line[start..end], style));
        }
    }

    spans
}

fn get_style(mode: &Mode, types: &[PositionType]) -> Style {
    match (
        mode,
        types.contains(&PositionType::CursorLine),
        types.contains(&PositionType::Cursor),
    ) {
        (Mode::Normal, true, true) => Style::default().add_modifier(Modifier::REVERSED),
        (Mode::Normal, true, false) => Style::default().bg(Color::DarkGray),
        (Mode::Command, true, _) => Style::default().bg(Color::DarkGray),
        (_, _, _) => Style::default(),
    }
}
