use ratatui::{
    prelude::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use teywi_keymap::action::Mode;

use crate::model::{Buffer, CursorPosition};

pub fn view(mode: &Mode, buffer: &Buffer, frame: &mut Frame, rect: Rect) {
    let lines = update_lines(mode, buffer);

    frame.render_widget(Paragraph::new(lines), rect);
}

fn update_lines<'a>(mode: &Mode, model: &'a Buffer) -> Vec<Line<'a>> {
    let mut lines = Vec::new();
    for (i, line) in model.lines.iter().enumerate() {
        lines.push(update_line(i, line, mode, model));
    }

    lines
}

fn update_line<'a>(index: usize, line: &'a str, mode: &Mode, model: &'a Buffer) -> Line<'a> {
    let style_expansion = get_style_expansions(index, line.chars().count(), mode, model);
    let mut spans = Vec::new();

    for (start, end, style) in style_expansion {
        spans.push(Span::styled(&line[start..end], style));
    }

    Line::from(spans)
}

#[derive(Debug, PartialEq)]
enum PositionType {
    Cursor,
    CursorLine,
    Default,
}

fn get_style_expansions(
    index: usize,
    length: usize,
    mode: &Mode,
    model: &Buffer,
) -> Vec<(usize, usize, Style)> {
    let mut positions = Vec::new();
    if model.cursor.line_number == index {
        let cursor_index = match &model.cursor.horizontial_position {
            CursorPosition::Absolute(i) => {
                if i >= &length {
                    length - 1
                } else {
                    i.clone()
                }
            }
            CursorPosition::_End => length - 1,
        };

        positions.push((0, PositionType::CursorLine));
        positions.push((length, PositionType::CursorLine));

        positions.push((cursor_index, PositionType::Cursor));
        positions.push((cursor_index + 1, PositionType::Cursor));
    } else {
        positions.push((0, PositionType::Default));
        positions.push((length, PositionType::Default));
    }

    let mut expansions = Vec::new();
    let mut last_position_index = None;
    let mut active_position_types = Vec::new();

    let sorted_positions = get_sorted_positions(positions);
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
            if !active_position_types.contains(&pt) {
                active_position_types.push(pt);
            } else {
                active_position_types.retain(|t| t != &pt);
            }
        }
    }

    expansions
}

fn get_sorted_positions(positions: Vec<(usize, PositionType)>) -> Vec<(usize, Vec<PositionType>)> {
    let mut result = Vec::new();
    for (index, position_type) in positions {
        if !result.iter().any(|(i, _)| i == &index) {
            result.push((index, vec![position_type]));
        } else {
            if let Some((_, position_types)) = result.iter_mut().find(|(i, _)| i == &index) {
                position_types.push(position_type);
            }
        }
    }

    result.sort_by(|(a, _), (b, _)| a.cmp(b));

    result
}

fn get_style(mode: &Mode, types: &Vec<PositionType>) -> Style {
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
