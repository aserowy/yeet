use yate_keymap::message::{CursorDirection, Mode};

use crate::model::buffer::{Buffer, CursorPosition};

pub fn update_by_direction(
    mode: &Mode,
    model: &mut Buffer,
    count: &usize,
    direction: &CursorDirection,
) {
    if model.lines.is_empty() {
        return;
    }

    if let Some(cursor) = &mut model.cursor {
        for _ in 0..*count {
            // TODO: replace all lines[..] calls with .get(..) everywhere
            match direction {
                CursorDirection::Bottom => {
                    cursor.vertical_index = model.lines.len() - 1;

                    let line_length = &model.lines[cursor.vertical_index].len();
                    let position = get_position(line_length, &cursor.horizontial_index);

                    cursor.horizontial_index = position;
                }
                CursorDirection::Down => {
                    if model.lines.len() - 1 > cursor.vertical_index {
                        cursor.vertical_index += 1;

                        let line_length = &model.lines[cursor.vertical_index].len();
                        let position = get_position(line_length, &cursor.horizontial_index);

                        cursor.horizontial_index = position;
                    }
                }
                CursorDirection::Left => {
                    let cursor_index = match cursor.horizontial_index {
                        CursorPosition::Absolute {
                            current,
                            expanded: _,
                        } => current,
                        CursorPosition::End => model.lines[cursor.vertical_index].len() - 1,
                        CursorPosition::None => return,
                    };

                    if cursor_index > 0 {
                        let next_index = cursor_index - 1;

                        cursor.horizontial_index = CursorPosition::Absolute {
                            current: next_index,
                            expanded: next_index,
                        };
                    }
                }
                CursorDirection::LineEnd => {
                    cursor.horizontial_index = CursorPosition::End;
                }
                CursorDirection::LineStart => {
                    cursor.horizontial_index = CursorPosition::Absolute {
                        current: 0,
                        expanded: 0,
                    };
                }
                CursorDirection::Right => {
                    let cursor_index = match cursor.horizontial_index {
                        CursorPosition::Absolute {
                            current,
                            expanded: _,
                        } => current,
                        CursorPosition::End => return,
                        CursorPosition::None => return,
                    };

                    let max_index = match mode {
                        Mode::Normal => model.lines[cursor.vertical_index].len() - 1,
                        Mode::Command => model.lines[cursor.vertical_index].len(),
                    };

                    if max_index > cursor_index {
                        let next_index = cursor_index + 1;

                        cursor.horizontial_index = CursorPosition::Absolute {
                            current: next_index,
                            expanded: next_index,
                        };
                    }
                }
                CursorDirection::Top => {
                    cursor.vertical_index = 0;

                    let line_length = &model.lines[cursor.vertical_index].len();
                    let position = get_position(line_length, &cursor.horizontial_index);

                    cursor.horizontial_index = position;
                }
                CursorDirection::Up => {
                    if cursor.vertical_index > 0 {
                        cursor.vertical_index -= 1;

                        let line_length = &model.lines[cursor.vertical_index].len();
                        let position = get_position(line_length, &cursor.horizontial_index);

                        cursor.horizontial_index = position;
                    }
                }
            }
        }
    }
}

fn get_position(line_length: &usize, position: &CursorPosition) -> CursorPosition {
    match position {
        CursorPosition::Absolute {
            current: _,
            expanded,
        } => {
            if expanded > line_length {
                CursorPosition::Absolute {
                    current: line_length - 1,
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
