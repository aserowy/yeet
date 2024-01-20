use ratatui::style::{Color, Modifier, Style};
use teywi_keymap::action::Mode;

use crate::model::buffer::{Buffer, CursorPosition};

#[derive(Debug, PartialEq)]
enum PositionType {
    Cursor,
    CursorLine,
    Default,
}

pub fn get_span_styles(
    index: usize,
    length: usize,
    mode: &Mode,
    model: &Buffer,
) -> Vec<(usize, usize, Style)> {
    let mut positions = vec![(0, PositionType::Default), (length, PositionType::Default)];
    positions.extend(get_cursor_line_positions(&index, &length, model));

    let sorted_positions = get_sorted_positions(positions);
    convert_sorted_positions_to_span_styles(mode, sorted_positions)
}

fn get_cursor_line_positions(
    index: &usize,
    length: &usize,
    model: &Buffer,
) -> Vec<(usize, PositionType)> {
    if &model.cursor.vertical_index == index {
        let cursor_index = match &model.cursor.horizontial_index {
            CursorPosition::Absolute(i) => {
                if i >= length {
                    length - 1
                } else {
                    *i
                }
            }
            CursorPosition::End => length - 1,
        };

        vec![
            (0, PositionType::CursorLine),
            (model.view_port.width, PositionType::CursorLine),
            (cursor_index, PositionType::Cursor),
            (cursor_index + 1, PositionType::Cursor),
        ]
    } else {
        vec![]
    }
}

fn get_sorted_positions(positions: Vec<(usize, PositionType)>) -> Vec<(usize, Vec<PositionType>)> {
    let mut result = Vec::new();
    for (index, position_type) in positions {
        if !result.iter().any(|(i, _)| i == &index) {
            result.push((index, vec![position_type]));
        } else if let Some((_, position_types)) = result.iter_mut().find(|(i, _)| i == &index) {
            position_types.push(position_type);
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
            if !active_position_types.contains(&pt) {
                active_position_types.push(pt);
            } else {
                active_position_types.retain(|t| t != &pt);
            }
        }
    }

    expansions
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
