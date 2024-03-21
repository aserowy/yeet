use crate::{
    message::{CursorDirection, LineDirection, TextModification},
    model::{undo::BufferChanged, Buffer, BufferLine, Cursor, CursorPosition, Mode, SearchModel},
};

use super::cursor;

pub fn update(
    mode: &Mode,
    search: &Option<SearchModel>,
    model: &mut Buffer,
    count: &usize,
    modification: &TextModification,
) -> Option<Vec<BufferChanged>> {
    match modification {
        TextModification::DeleteLine => {
            if model.lines.is_empty() {
                None
            } else if let Some(cursor) = &mut model.cursor {
                let mut changes = Vec::new();
                for _ in 0..*count {
                    let line_index = cursor.vertical_index;
                    let line = model.lines.remove(line_index);
                    let content = line.content.to_string();

                    let line_count = model.lines.len();
                    if line_count == 0 {
                        cursor.vertical_index = 0;
                    } else if line_index >= line_count {
                        cursor.vertical_index = line_count - 1;
                    }

                    changes.push(BufferChanged::LineRemoved(line_index, content));
                }

                cursor::validate(mode, model);

                Some(changes)
            } else {
                None
            }
        }
        TextModification::DeleteMotion(delete_count, motion) => {
            let cursor = &model.cursor;
            let pre_motion_cursor = match cursor.clone() {
                Some(it) => it,
                None => return None,
            };

            for _ in 0..*count {
                cursor::update_by_direction(mode, search, model, delete_count, motion);
            }

            let post_motion_cursor = match &model.cursor {
                Some(it) => it,
                None => return None,
            };

            let mut changes = Vec::new();
            if is_line_delete(motion) {
                let pre_index = pre_motion_cursor.vertical_index;
                let post_index = post_motion_cursor.vertical_index;

                if pre_index == post_index {
                    return None;
                }

                let count = if pre_index > post_index {
                    pre_index - post_index + 1
                } else {
                    model.cursor = Some(pre_motion_cursor.clone());
                    post_index - pre_index + 1
                };

                let action = &TextModification::DeleteLine;
                if let Some(cng) = update(mode, search, model, &count, action) {
                    changes.extend(cng);
                }
            } else {
                // TODO: multi line motion like search
                let line = match model.lines.get_mut(pre_motion_cursor.vertical_index) {
                    Some(it) => it,
                    None => return None,
                };

                let pre_index = get_cursor_index(&pre_motion_cursor, line);
                let post_index = get_cursor_index(post_motion_cursor, line);

                let (index, mut count) = if pre_index > post_index {
                    (post_index, pre_index - post_index)
                } else {
                    model.cursor = Some(pre_motion_cursor.clone());
                    (pre_index, post_index - pre_index)
                };

                if is_inclusive(motion) {
                    count += 1;
                }

                let new: String = line
                    .content
                    .chars()
                    .enumerate()
                    .filter_map(|(i, c)| {
                        if i >= index && i < index + count {
                            None
                        } else {
                            Some(c)
                        }
                    })
                    .collect();

                let changed = BufferChanged::Content(
                    pre_motion_cursor.vertical_index,
                    line.content.to_string(),
                    new.to_string(),
                );

                line.content = new;

                changes.push(changed);
            }

            cursor::validate(mode, model);

            Some(changes)
        }
        TextModification::Insert(raw) => {
            let line = get_line_or_create_on_empty(model);
            if let Some((cursor, line)) = line {
                let index = get_cursor_index(cursor, line);

                let next_index = index + raw.chars().count();
                cursor.horizontal_index = CursorPosition::Absolute {
                    current: next_index,
                    expanded: next_index,
                };

                let new = format!(
                    "{}{}{}",
                    &line.content[..index],
                    raw,
                    &line.content[index..]
                );

                let changed = BufferChanged::Content(
                    cursor.vertical_index,
                    line.content.to_string(),
                    new.to_string(),
                );

                line.content = new;

                Some(vec![changed])
            } else {
                None
            }
        }
        TextModification::InsertNewLine(direction) => {
            if let Some(cursor) = &mut model.cursor {
                let index = match direction {
                    LineDirection::Up => cursor.vertical_index,
                    LineDirection::Down => {
                        if model.lines.is_empty() {
                            cursor.vertical_index = 0;

                            0
                        } else {
                            let index = cursor.vertical_index + 1;
                            cursor.vertical_index = index;

                            index
                        }
                    }
                };

                cursor.horizontal_index = CursorPosition::Absolute {
                    current: 0,
                    expanded: 0,
                };

                model.lines.insert(index, BufferLine::default());

                Some(vec![BufferChanged::LineAdded(index, "".to_string())])
            } else {
                None
            }
        }
    }
}

fn is_inclusive(motion: &CursorDirection) -> bool {
    match motion {
        CursorDirection::Left
        | CursorDirection::Right
        | CursorDirection::LineStart
        | CursorDirection::Search(_)
        | CursorDirection::Up
        | CursorDirection::Down
        | CursorDirection::Bottom
        | CursorDirection::Top => false,

        CursorDirection::FindBackward(_)
        | CursorDirection::FindForward(_)
        | CursorDirection::TillBackward(_)
        | CursorDirection::TillForward(_)
        | CursorDirection::LineEnd => true,
    }
}

fn is_line_delete(motion: &CursorDirection) -> bool {
    match motion {
        CursorDirection::Up
        | CursorDirection::Down
        | CursorDirection::Bottom
        | CursorDirection::Top => true,

        CursorDirection::FindBackward(_)
        | CursorDirection::FindForward(_)
        | CursorDirection::TillBackward(_)
        | CursorDirection::TillForward(_)
        | CursorDirection::Left
        | CursorDirection::Right
        | CursorDirection::Search(_)
        | CursorDirection::LineEnd
        | CursorDirection::LineStart => false,
    }
}

fn get_line_or_create_on_empty(model: &mut Buffer) -> Option<(&mut Cursor, &mut BufferLine)> {
    if let Some(cursor) = &mut model.cursor {
        if cursor.horizontal_index == CursorPosition::None {
            return None;
        }

        if model.lines.is_empty() {
            cursor.vertical_index = 0;

            let line = BufferLine::default();
            model.lines.push(line);

            Some((cursor, &mut model.lines[0]))
        } else {
            let line = match model.lines.get_mut(cursor.vertical_index) {
                Some(it) => it,
                None => return None,
            };

            Some((cursor, line))
        }
    } else {
        None
    }
}

fn get_cursor_index(cursor: &Cursor, line: &BufferLine) -> usize {
    match cursor.horizontal_index {
        CursorPosition::Absolute {
            current,
            expanded: _,
        } => current,
        CursorPosition::End => {
            if line.is_empty() {
                0
            } else {
                line.len() - 1
            }
        }
        CursorPosition::None => unreachable!(),
    }
}
