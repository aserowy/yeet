use yeet_keymap::message::{CursorDirection, Mode};

use crate::model::buffer::{Buffer, BufferLine, Cursor, CursorPosition};

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

                let position = get_position(mode, &line.len(), &cursor.horizontial_index);
                cursor.horizontial_index = position;
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
                let position = get_position(mode, line_length, &cursor.horizontial_index);

                cursor.horizontial_index = position;
            }
            CursorDirection::FindBackward(find) => {
                if let Some(found) = find_char_backwards(find, &model.lines, cursor) {
                    cursor.horizontial_index = CursorPosition::Absolute {
                        current: found,
                        expanded: found,
                    };
                }
            }
            CursorDirection::FindForward(find) => {
                if let Some(found) = find_char_forward(find, &model.lines, cursor) {
                    cursor.horizontial_index = CursorPosition::Absolute {
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

                let index = match get_horizontal_index(&cursor.horizontial_index, line) {
                    Some(index) => index,
                    None => return,
                };

                if index > 0 {
                    let next_index = index - 1;

                    cursor.horizontial_index = CursorPosition::Absolute {
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

                    cursor.horizontial_index = CursorPosition::Absolute {
                        current: max_index,
                        expanded: max_index,
                    };
                } else {
                    cursor.horizontial_index = CursorPosition::End;
                }
            }
            CursorDirection::LineStart => {
                cursor.horizontial_index = CursorPosition::Absolute {
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

                let cursor_index = match cursor.horizontial_index {
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

                    cursor.horizontial_index = CursorPosition::Absolute {
                        current: next_index,
                        expanded: next_index,
                    };
                }
            }
            CursorDirection::TillBackward(find) => {
                if let Some(found) = find_char_backwards(find, &model.lines, cursor) {
                    let new = found + 1;
                    cursor.horizontial_index = CursorPosition::Absolute {
                        current: new,
                        expanded: new,
                    };
                }
            }
            CursorDirection::TillForward(find) => {
                if let Some(found) = find_char_forward(find, &model.lines, cursor) {
                    let new = found - 1;
                    cursor.horizontial_index = CursorPosition::Absolute {
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
                let position = get_position(mode, line_length, &cursor.horizontial_index);

                cursor.horizontial_index = position;
            }
            CursorDirection::Up => {
                if cursor.vertical_index > 0 {
                    cursor.vertical_index -= 1;

                    let line = match model.lines.get(cursor.vertical_index) {
                        Some(line) => line,
                        None => return,
                    };

                    let line_length = &line.len();
                    let position = get_position(mode, line_length, &cursor.horizontial_index);

                    cursor.horizontial_index = position;
                }
            }
        }
    }
}

fn find_char_forward(find: &char, lines: &[BufferLine], cursor: &mut Cursor) -> Option<usize> {
    let line = match lines.get(cursor.vertical_index) {
        Some(line) => line,
        None => return None,
    };

    let index = match get_horizontal_index(&cursor.horizontial_index, line) {
        Some(index) => index,
        None => return None,
    };

    let find = line
        .content
        .chars()
        .skip(index + 1)
        .position(|c| &c == find);

    if let Some(found) = find {
        Some(index + found + 1)
    } else {
        None
    }
}

fn find_char_backwards(find: &char, lines: &[BufferLine], cursor: &Cursor) -> Option<usize> {
    let line = match lines.get(cursor.vertical_index) {
        Some(line) => line,
        None => return None,
    };

    let index = match get_horizontal_index(&cursor.horizontial_index, line) {
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
            let index = if line.len() == 0 { 0 } else { line.len() - 1 };
            Some(index)
        }
        CursorPosition::None => None,
    }
}

pub fn validate(mode: &Mode, model: &mut Buffer) {
    if let Some(cursor) = &mut model.cursor {
        let position = if model.lines.is_empty() {
            get_position(mode, &0, &cursor.horizontial_index)
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
            get_position(mode, line_length, &cursor.horizontial_index)
        };

        cursor.horizontial_index = position;
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
        Mode::Command => 0,
        Mode::Insert => 0,
        Mode::Navigation => 1,
        Mode::Normal => 1,
    }
}
