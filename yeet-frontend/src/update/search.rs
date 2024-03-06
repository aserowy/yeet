use std::cmp::Ordering;

use ratatui::style::Color;
use yeet_keymap::message::{CommandMode, Mode};

use crate::{
    action::Action,
    model::{
        buffer::{Buffer, CursorPosition, StylePartial, StylePartialSpan},
        Model,
    },
};

use super::model::preview;

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

// TODO: refactor
pub fn select(model: &mut Model) -> Option<Vec<Action>> {
    let downwards = matches!(model.mode, Mode::Command(CommandMode::SearchDown));
    let cursor = model.current.buffer.cursor.as_mut()?;
    if cursor.horizontal_index == CursorPosition::None {
        return None;
    }

    let vertical_index = cursor.vertical_index;
    let mut enumeration: Vec<_> = model
        .current
        .buffer
        .lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.search.is_some())
        .collect();

    enumeration.sort_unstable_by(|(current, _), (cmp, _)| {
        sort_by_index(*current, *cmp, vertical_index, downwards)
    });

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
                let index_cmp = current <= &start;
                if downwards ^ index_cmp {
                    continue;
                }
            }
        }

        cursor.vertical_index = i;
        cursor.horizontal_index = CursorPosition::Absolute {
            current: start,
            expanded: start,
        };

        break;
    }

    let mut actions = Vec::new();
    if let Some(preview_actions) = preview::path(model, true, true) {
        actions.extend(preview_actions);
        model.preview.buffer.lines.clear();
        preview::viewport(model);
    }

    if actions.is_empty() {
        None
    } else {
        Some(actions)
    }
}

fn sort_by_index(current: usize, cmp: usize, index: usize, downwards: bool) -> Ordering {
    if current == cmp {
        return Ordering::Equal;
    }

    if downwards {
        if current >= index {
            if current > cmp {
                if cmp >= index {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            } else {
                Ordering::Less
            }
        } else {
            current.cmp(&cmp)
        }
    } else if current <= index {
        if current > cmp {
            Ordering::Less
        } else if cmp <= index {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    } else if current > cmp {
        if cmp <= index {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    } else {
        Ordering::Greater
    }
}

mod test {
    #[test]
    fn sort_by_index_downwards() {
        let vertical = 5;
        let mut sorted = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        sorted.sort_by(|i, j| super::sort_by_index(*i, *j, vertical, true));

        assert_eq!(vec![5, 6, 7, 8, 9, 0, 1, 2, 3, 4], sorted);
    }

    #[test]
    fn sort_by_index_upwards() {
        let vertical = 5;
        let mut sorted = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        sorted.sort_by(|i, j| super::sort_by_index(*i, *j, vertical, false));

        assert_eq!(vec![5, 4, 3, 2, 1, 0, 9, 8, 7, 6], sorted);
    }
}
