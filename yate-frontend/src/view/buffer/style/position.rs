use ratatui::style::{Color, Modifier, Style};
use yate_keymap::message::Mode;

use super::StyleSpan;

#[derive(Clone, Debug, PartialEq)]
pub enum PositionType {
    Cursor,
    CursorLine,
    Default,
    LineNumberAbsolute,
    LineNumberRelative,
}

pub type StylePosition = (usize, PositionType);
pub type StylePositionByLineIndex = (usize, Vec<StylePosition>);
pub type StylePositionGroup = (usize, Vec<PositionType>);

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
