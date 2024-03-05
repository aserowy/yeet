use ratatui::style::Color;
use yeet_keymap::message::{CommandMode, Mode};

use crate::{
    action::Action,
    model::{
        buffer::{Buffer, CursorPosition, StylePartial, StylePartialSpan},
        Model,
    },
};

// TODO: n, N, enter, save in reg (add reg types?)
pub fn update(model: &mut Model) {
    let search = match model.commandline.buffer.lines.last() {
        Some(line) => &line.content,
        None => return,
    };

    if model.parent.path.is_some() {
        set_styles(&mut model.parent.buffer, search);
    }

    if model.preview.path.is_dir() {
        set_styles(&mut model.preview.buffer, search);
    }

    set_styles(&mut model.current.buffer, search);
}

pub fn clear(model: &mut Model) {
    for line in &mut model.parent.buffer.lines {
        line.search = None;
    }
    for line in &mut model.preview.buffer.lines {
        line.search = None;
    }
    for line in &mut model.current.buffer.lines {
        line.search = None;
    }
}

fn set_styles(buffer: &mut Buffer, search: &str) {
    let len = search.chars().count();
    for line in &mut buffer.lines {
        line.search = None;

        // TODO: smart search
        let start = match line.content.find(search) {
            Some(it) => line.content[..it].chars().count(),
            None => continue,
        };

        line.search = Some(vec![
            StylePartialSpan {
                start,
                end: start + len,
                style: StylePartial::Foreground(Color::DarkGray),
            },
            StylePartialSpan {
                start,
                end: start + len,
                style: StylePartial::Background(Color::Magenta),
            },
        ]);
    }
}

pub fn select(model: &mut Model) -> Option<Vec<Action>> {
    let downwards = matches!(model.mode, Mode::Command(CommandMode::SearchDown));
    let cursor = model.current.buffer.cursor.as_mut()?;
    if cursor.horizontal_index == CursorPosition::None {
        return None;
    }

    let vertical_index = cursor.vertical_index;
    let enumeration = model
        .current
        .buffer
        .lines
        .iter()
        .enumerate()
        .filter(|(i, l)| {
            if l.search.is_none() {
                return false;
            }

            if downwards {
                i >= &vertical_index
            } else {
                i <= &vertical_index
            }
        });

    // TODO: upward search
    for (i, line) in enumeration {
        let start = match &line.search {
            Some(it) => match it.first() {
                Some(s) => s.start,
                None => continue,
            },
            None => continue,
        };

        if i == vertical_index {
            if let CursorPosition::Absolute { current, .. } = &cursor.horizontal_index {
                if current > &start {
                    continue;
                }
            }
        }

        cursor.vertical_index = i;
        cursor.horizontal_index = CursorPosition::Absolute {
            current: start,
            expanded: start,
        };

        // TODO: return actions to refresh preview?

        break;
    }

    None
}
