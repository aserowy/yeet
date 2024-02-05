use yate_keymap::message::{NewLineDirection, TextModification};

use crate::model::buffer::{undo::BufferChanged, Buffer, BufferLine, Cursor, CursorPosition};

pub fn update(model: &mut Buffer, modification: &TextModification) -> Option<Vec<BufferChanged>> {
    // TODO: most None must return Some(Vec<BufferChanged>) instead
    match modification {
        TextModification::DeleteCharBeforeCursor => {
            let line = get_line(model);
            if let Some((cursor, line)) = line {
                let index = get_cursor_index(cursor, line);
                if index > 0 {
                    let next_index = index - 1;
                    cursor.horizontial_index = CursorPosition::Absolute {
                        current: next_index,
                        expanded: next_index,
                    };

                    let pre = &line.content[..index - 1];
                    let post = &line.content[index..];

                    let new = format!("{}{}", pre, post);
                    let changed = BufferChanged::Content(
                        cursor.vertical_index,
                        line.content.to_string(),
                        new.to_string(),
                    );

                    line.content = new;

                    Some(vec![changed])
                } else {
                    // TODO: char before cursor removes empty line and inserts rest to above
                    None
                }
            } else {
                None
            }
        }
        TextModification::DeleteCharOnCursor => {
            let line = get_line(model);
            if let Some((cursor, line)) = line {
                let index = get_cursor_index(cursor, line);
                if index < line.len() {
                    let pre = &line.content[..index];
                    let post = &line.content[index + 1..];

                    let new = format!("{}{}", pre, post);
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
            } else {
                None
            }
        }
        TextModification::DeleteLineOnCursor => {
            if model.lines.is_empty() {
                None
            } else if let Some(cursor) = &mut model.cursor {
                let line_index = cursor.vertical_index;
                let line = model.lines.remove(line_index);
                let content = line.content.to_string();

                let line_count = model.lines.len();
                if line_count >= line_index {
                    cursor.vertical_index = line_count - 1;
                }

                Some(vec![BufferChanged::LineRemoved(line_index, content)])
            } else {
                None
            }
        }
        TextModification::Insert(raw) => {
            let line = get_line(model);
            if let Some((cursor, line)) = line {
                let index = get_cursor_index(cursor, line);

                let next_index = index + raw.chars().count();
                cursor.horizontial_index = CursorPosition::Absolute {
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
                    NewLineDirection::Above => cursor.vertical_index,
                    NewLineDirection::Under => {
                        let index = cursor.vertical_index + 1;
                        cursor.vertical_index = index;

                        index
                    }
                };

                model.lines.insert(index, BufferLine::default());

                Some(vec![BufferChanged::LineAdded(index, "".to_string())])
            } else {
                None
            }
        }
    }
}

fn get_line(model: &mut Buffer) -> Option<(&mut Cursor, &mut BufferLine)> {
    if let Some(cursor) = &mut model.cursor {
        if cursor.horizontial_index == CursorPosition::None {
            return None;
        }

        if model.lines.is_empty() {
            cursor.vertical_index = 0;

            let line = BufferLine::default();
            model.lines.push(line);

            Some((cursor, &mut model.lines[0]))
        } else {
            let line_index = cursor.vertical_index;

            Some((cursor, &mut model.lines[line_index]))
        }
    } else {
        None
    }
}

fn get_cursor_index(cursor: &Cursor, line: &BufferLine) -> usize {
    match cursor.horizontial_index {
        CursorPosition::Absolute {
            current,
            expanded: _,
        } => current,
        // FIX: count > 0 checks
        CursorPosition::End => line.len() - 1,
        CursorPosition::None => unreachable!(),
    }
}
