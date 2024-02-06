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
                    let position = get_position(mode, line_length, &cursor.horizontial_index);

                    cursor.horizontial_index = position;
                }
                CursorDirection::Down => {
                    let max_index = model.lines.len() - 1;
                    if cursor.vertical_index >= max_index {
                        cursor.vertical_index = max_index;
                    } else {
                        cursor.vertical_index += 1
                    }

                    let line_length = &model.lines[cursor.vertical_index].len();
                    let position = get_position(mode, line_length, &cursor.horizontial_index);

                    cursor.horizontial_index = position;
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
                    if mode == &Mode::Insert {
                        let index_correction = get_index_correction(mode);
                        let max_index = model.lines[cursor.vertical_index].len() - index_correction;

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
                    let index_correction = get_index_correction(mode);
                    let max_index = model.lines[cursor.vertical_index].len() - index_correction;

                    let cursor_index = match cursor.horizontial_index {
                        CursorPosition::Absolute {
                            current,
                            expanded: _,
                        } => current,
                        CursorPosition::End => {
                            if mode == &Mode::Insert {
                                // NOTE: -1 to trigger the replacement with absolute cursor
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
                CursorDirection::Top => {
                    cursor.vertical_index = 0;

                    let line_length = &model.lines[cursor.vertical_index].len();
                    let position = get_position(mode, line_length, &cursor.horizontial_index);

                    cursor.horizontial_index = position;
                }
                CursorDirection::Up => {
                    if cursor.vertical_index > 0 {
                        cursor.vertical_index -= 1;

                        let line_length = &model.lines[cursor.vertical_index].len();
                        let position = get_position(mode, line_length, &cursor.horizontial_index);

                        cursor.horizontial_index = position;
                    }
                }
                CursorDirection::Validate => {
                    let max_index = model.lines.len() - 1;
                    if cursor.vertical_index >= max_index {
                        cursor.vertical_index = max_index;
                    }

                    let line_length = &model.lines[cursor.vertical_index].len();
                    let position = get_position(mode, line_length, &cursor.horizontial_index);

                    cursor.horizontial_index = position;
                }
            }
        }
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
