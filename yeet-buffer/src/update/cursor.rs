use std::cmp::Ordering;

use crate::{
    message::{CursorDirection, SearchDirection},
    model::{Buffer, BufferLine, Cursor, CursorPosition, Mode},
};

// TODO: refactor
pub fn update_by_direction(
    mode: &Mode,
    model: &mut Buffer,
    count: &usize,
    direction: &CursorDirection,
) {
    if model.lines.is_empty() {
        return;
    }

    let cursor = match &mut model.cursor {
        Some(cursor) => cursor,
        None => return,
    };

    for _ in 0..*count {
        match direction {
            CursorDirection::Bottom => {
                cursor.vertical_index = model.lines.len() - 1;
                let line = match model.lines.get(cursor.vertical_index) {
                    Some(line) => line,
                    None => return,
                };

                let position = get_position(mode, &line.len(), &cursor.horizontal_index);
                cursor.horizontal_index = position;
            }
            CursorDirection::Down => {
                let max_index = model.lines.len() - 1;
                if cursor.vertical_index >= max_index {
                    cursor.vertical_index = max_index;
                } else {
                    cursor.vertical_index += 1
                }

                let line = match model.lines.get(cursor.vertical_index) {
                    Some(line) => line,
                    None => return,
                };

                let line_length = &line.len();
                let position = get_position(mode, line_length, &cursor.horizontal_index);

                cursor.horizontal_index = position;
            }
            CursorDirection::FindBackward(find) => {
                if let Some(found) = find_char_backward(find, &model.lines, cursor) {
                    cursor.horizontal_index = CursorPosition::Absolute {
                        current: found,
                        expanded: found,
                    };
                }
            }
            CursorDirection::FindForward(find) => {
                if let Some(found) = find_char_forward(find, &model.lines, cursor) {
                    cursor.horizontal_index = CursorPosition::Absolute {
                        current: found,
                        expanded: found,
                    };
                }
            }
            CursorDirection::Left => {
                let line = match model.lines.get(cursor.vertical_index) {
                    Some(line) => line,
                    None => return,
                };

                let index = match get_horizontal_index(&cursor.horizontal_index, line) {
                    Some(index) => index,
                    None => return,
                };

                if index > 0 {
                    let next_index = index - 1;

                    cursor.horizontal_index = CursorPosition::Absolute {
                        current: next_index,
                        expanded: next_index,
                    };
                }
            }
            CursorDirection::LineEnd => {
                if mode == &Mode::Insert {
                    let line = match model.lines.get(cursor.vertical_index) {
                        Some(line) => line,
                        None => return,
                    };

                    let index_correction = get_index_correction(mode);
                    let max_index = line.len() - index_correction;

                    cursor.horizontal_index = CursorPosition::Absolute {
                        current: max_index,
                        expanded: max_index,
                    };
                } else {
                    cursor.horizontal_index = CursorPosition::End;
                }
            }
            CursorDirection::LineStart => {
                cursor.horizontal_index = CursorPosition::Absolute {
                    current: 0,
                    expanded: 0,
                };
            }
            CursorDirection::Right => {
                let line = match model.lines.get(cursor.vertical_index) {
                    Some(line) => line,
                    None => return,
                };

                let index_correction = get_index_correction(mode);
                let max_index = line.len() - index_correction;

                let cursor_index = match cursor.horizontal_index {
                    CursorPosition::Absolute {
                        current,
                        expanded: _,
                    } => current,
                    CursorPosition::End => {
                        if mode == &Mode::Insert {
                            // NOTE: -1 to trigger replacement with absolute cursor
                            max_index - 1
                        } else {
                            return;
                        }
                    }
                    CursorPosition::None => return,
                };

                if max_index > cursor_index {
                    let next_index = cursor_index + 1;

                    cursor.horizontal_index = CursorPosition::Absolute {
                        current: next_index,
                        expanded: next_index,
                    };
                }
            }
            CursorDirection::Search(direction) => {
                select(cursor, &model.lines, direction);
            }
            CursorDirection::TillBackward(find) => {
                if let Some(found) = find_char_backward(find, &model.lines, cursor) {
                    let new = found + 1;
                    cursor.horizontal_index = CursorPosition::Absolute {
                        current: new,
                        expanded: new,
                    };
                }
            }
            CursorDirection::TillForward(find) => {
                if let Some(found) = find_char_forward(find, &model.lines, cursor) {
                    let new = found - 1;
                    cursor.horizontal_index = CursorPosition::Absolute {
                        current: new,
                        expanded: new,
                    };
                }
            }
            CursorDirection::Top => {
                cursor.vertical_index = 0;

                let line = match model.lines.get(cursor.vertical_index) {
                    Some(line) => line,
                    None => return,
                };

                let line_length = &line.len();
                let position = get_position(mode, line_length, &cursor.horizontal_index);

                cursor.horizontal_index = position;
            }
            CursorDirection::Up => {
                if cursor.vertical_index > 0 {
                    cursor.vertical_index -= 1;

                    let line = match model.lines.get(cursor.vertical_index) {
                        Some(line) => line,
                        None => return,
                    };

                    let line_length = &line.len();
                    let position = get_position(mode, line_length, &cursor.horizontal_index);

                    cursor.horizontal_index = position;
                }
            }
        }
    }
}

fn find_char_forward(find: &char, lines: &[BufferLine], cursor: &mut Cursor) -> Option<usize> {
    let current = match lines.get(cursor.vertical_index) {
        Some(line) => line,
        None => return None,
    };

    let index = match get_horizontal_index(&cursor.horizontal_index, current) {
        Some(index) => index,
        None => return None,
    };

    current
        .content
        .chars()
        .skip(index + 1)
        .position(|c| &c == find)
        .map(|i| index + i + 1)
}

fn find_char_backward(find: &char, lines: &[BufferLine], cursor: &Cursor) -> Option<usize> {
    let line = match lines.get(cursor.vertical_index) {
        Some(line) => line,
        None => return None,
    };

    let index = match get_horizontal_index(&cursor.horizontal_index, line) {
        Some(index) => index,
        None => return None,
    };

    if index <= 1 {
        return None;
    }

    line.content
        .chars()
        .take(index)
        .collect::<Vec<_>>()
        .iter()
        .rposition(|c| c == find)
}

fn get_horizontal_index(horizontial_index: &CursorPosition, line: &BufferLine) -> Option<usize> {
    match horizontial_index {
        CursorPosition::Absolute {
            current,
            expanded: _,
        } => Some(*current),
        CursorPosition::End => {
            let index = if line.is_empty() { 0 } else { line.len() - 1 };
            Some(index)
        }
        CursorPosition::None => None,
    }
}

pub fn validate(mode: &Mode, model: &mut Buffer) {
    if let Some(cursor) = &mut model.cursor {
        let position = if model.lines.is_empty() {
            get_position(mode, &0, &cursor.horizontal_index)
        } else {
            let max_index = model.lines.len() - 1;
            if cursor.vertical_index >= max_index {
                cursor.vertical_index = max_index;
            }

            let line = match model.lines.get(cursor.vertical_index) {
                Some(line) => line,
                None => return,
            };

            let line_length = &line.len();
            get_position(mode, line_length, &cursor.horizontal_index)
        };

        cursor.horizontal_index = position;
    }
}

fn get_position(mode: &Mode, line_length: &usize, position: &CursorPosition) -> CursorPosition {
    match position {
        CursorPosition::Absolute {
            current: _,
            expanded,
        } => {
            let index_correction = get_index_correction(mode);
            let max_length = if line_length == &0 {
                index_correction
            } else {
                line_length - index_correction
            };

            if expanded > &max_length {
                CursorPosition::Absolute {
                    current: max_length,
                    expanded: *expanded,
                }
            } else {
                CursorPosition::Absolute {
                    current: *expanded,
                    expanded: *expanded,
                }
            }
        }
        CursorPosition::End => CursorPosition::End,
        CursorPosition::None => CursorPosition::None,
    }
}

fn get_index_correction(mode: &Mode) -> usize {
    match mode {
        Mode::Command(_) => 0,
        Mode::Insert => 0,
        Mode::Navigation => 1,
        Mode::Normal => 1,
    }
}

fn select(cursor: &mut Cursor, lines: &[BufferLine], direction: &SearchDirection) {
    if cursor.horizontal_index == CursorPosition::None {
        return;
    }

    let vertical_index = cursor.vertical_index;
    let mut enumeration: Vec<_> = lines
        .iter()
        .enumerate()
        .filter(|(_, bl)| bl.search.is_some())
        .collect();

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
        use crate::message::SearchDirection;

        let vertical = 5;
        let mut sorted = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        sorted.sort_by(|i, j| super::sort_by_index(*i, *j, vertical, &SearchDirection::Down));

        assert_eq!(vec![5, 6, 7, 8, 9, 0, 1, 2, 3, 4], sorted);
    }

    #[test]
    fn sort_by_index_upward() {
        use crate::message::SearchDirection;

        let vertical = 5;
        let mut sorted = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        sorted.sort_by(|i, j| super::sort_by_index(*i, *j, vertical, &SearchDirection::Up));

        assert_eq!(vec![5, 4, 3, 2, 1, 0, 9, 8, 7, 6], sorted);
    }
}
