use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
};
use yate_keymap::message::Mode;

use crate::model::buffer::ViewPort;

pub mod cursor;
pub mod line_number;

pub type StylePosition = (usize, PositionType);
type StylePositionByLineIndex = (usize, Vec<StylePosition>);
type StylePositionGroup = (usize, Vec<PositionType>);
type StyleSpan = (usize, usize, Style);

#[derive(Clone, Debug, PartialEq)]
pub enum PositionType {
    Cursor,
    CursorLine,
    Default,
    LineNumberAbsolute,
    LineNumberRelative,
}

pub fn get_sorted_positions(positions: Vec<StylePosition>) -> Vec<StylePositionGroup> {
    let mut result: Vec<StylePositionGroup> = Vec::new();
    for (index, position_type) in positions {
        if let Some((_, position_types)) = result.iter_mut().find(|(i, _)| i == &index) {
            position_types.push(position_type.clone());
        } else {
            result.push((index, vec![position_type.clone()]));
        }
    }

    result.sort_by(|(a, _), (b, _)| a.cmp(b));

    result
}

pub fn convert_sorted_positions_to_span_styles(
    mode: &Mode,
    sorted_positions: Vec<StylePositionGroup>,
) -> Vec<StyleSpan> {
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

fn get_style(mode: &Mode, types: &[PositionType]) -> Style {
    if types
        .iter()
        .any(|tp| tp == &PositionType::LineNumberRelative)
    {
        return Style::default().fg(Color::DarkGray);
    }

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
