use std::cmp::Ordering;

use ratatui::style::Color;
use yeet_keymap::message::{CommandMode, Mode, SearchDirection};

use crate::{
    action::Action,
    model::{
        buffer::{Buffer, CursorPosition, StylePartial, StylePartialSpan},
        Model, SearchModel,
    },
};

use super::model::preview;

pub fn update(model: &mut Model) {
    let search = match model.commandline.buffer.lines.last() {
        Some(line) => &line.content,
        None => return,
    };

    let direction = if let Mode::Command(CommandMode::Search(direction)) = &model.mode {
        direction.clone()
    } else {
        return;
    };

    model.commandline.search = Some(SearchModel {
        last: search.to_owned(),
        direction,
    });

    super::search::search(model);
}

fn search(model: &mut Model) {
    let search = match &model.commandline.search {
        Some(it) => it.last.as_str(),
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
    let smart_case = search.chars().all(|c| c.is_ascii_lowercase());

    for line in &mut buffer.lines {
        line.search = None;

        let mut content = line.content.as_str();

        let lower = content.to_lowercase();
        if smart_case {
            content = lower.as_str();
        };

        let start = match content.find(search) {
            Some(it) => content[..it].chars().count(),
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

pub fn search_and_select(model: &mut Model, is_next: bool) -> Option<Vec<Action>> {
    search(model);

    let search_model = match &model.commandline.search {
        Some(it) => it,
        None => return None,
    };

    select(search_model, &mut model.current.buffer, is_next);

    if let Some(preview_actions) = preview::path(model, true, true) {
        model.preview.buffer.lines.clear();
        preview::viewport(model);
        Some(preview_actions)
    } else {
        None
    }
}

pub fn select(model: &SearchModel, buffer: &mut Buffer, is_next: bool) {
    let cursor = match buffer.cursor.as_mut() {
        Some(it) => it,
        None => return,
    };

    if cursor.horizontal_index == CursorPosition::None {
        return;
    }

    let vertical_index = cursor.vertical_index;
    let mut enumeration: Vec<_> = buffer
        .lines
        .iter()
        .enumerate()
        .filter(|(_, bl)| bl.search.is_some())
        .collect();

    let direction = if is_next {
        &model.direction
    } else {
        match model.direction {
            SearchDirection::Down => &SearchDirection::Up,
            SearchDirection::Up => &SearchDirection::Down,
        }
    };

    enumeration.sort_unstable_by(|(current, _), (cmp, _)| {
        sort_by_index(*current, *cmp, vertical_index, direction)
    });

    for (i, line) in enumeration {
        let start = match &line.search {
            Some(it) => match it.first() {
                Some(s) => s.start,
                None => continue,
            },
            None => continue,
        };

        let downward = direction == &SearchDirection::Down;
        if i == vertical_index {
            if let CursorPosition::Absolute { current, .. } = &cursor.horizontal_index {
                if downward && current >= &start || !downward && current <= &start {
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
}

fn sort_by_index(
    current: usize,
    cmp: usize,
    index: usize,
    direction: &SearchDirection,
) -> Ordering {
    let downward = direction == &SearchDirection::Down;
    if current == cmp {
        return Ordering::Equal;
    }

    if downward {
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
    fn sort_by_index_downward() {
        use yeet_keymap::message::SearchDirection;

        let vertical = 5;
        let mut sorted = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        sorted.sort_by(|i, j| super::sort_by_index(*i, *j, vertical, &SearchDirection::Down));

        assert_eq!(vec![5, 6, 7, 8, 9, 0, 1, 2, 3, 4], sorted);
    }

    #[test]
    fn sort_by_index_upward() {
        use yeet_keymap::message::SearchDirection;

        let vertical = 5;
        let mut sorted = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        sorted.sort_by(|i, j| super::sort_by_index(*i, *j, vertical, &SearchDirection::Up));

        assert_eq!(vec![5, 4, 3, 2, 1, 0, 9, 8, 7, 6], sorted);
    }
}
