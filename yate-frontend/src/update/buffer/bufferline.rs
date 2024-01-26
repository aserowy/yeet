use yate_keymap::message::TextModification;

use crate::model::buffer::{Buffer, BufferLine, CursorPosition};

pub fn update(model: &mut Buffer, modification: &TextModification) {
    if let Some(cursor) = &mut model.cursor {
        if cursor.horizontial_index == CursorPosition::None {
            return;
        }

        let line = if model.lines.is_empty() {
            cursor.vertical_index = 0;

            let line = BufferLine::default();
            model.lines.push(line);

            &mut model.lines[0]
        } else {
            &mut model.lines[cursor.vertical_index]
        };

        let index = match cursor.horizontial_index {
            CursorPosition::Absolute(n) => n,
            // FIX: count > 0 checks
            CursorPosition::End => line.content.chars().count() - 1,
            CursorPosition::None => unreachable!(),
        };

        match modification {
            TextModification::DeleteCharOnCursor => {
                if index == 0 {
                    return;
                }

                cursor.horizontial_index = CursorPosition::Absolute(index - 1);

                line.content = format!("{}{}", &line.content[..index - 1], &line.content[index..]);
            }
            TextModification::Insert(raw) => {
                cursor.horizontial_index = CursorPosition::Absolute(index + raw.chars().count());

                line.content = format!(
                    "{}{}{}",
                    &line.content[..index],
                    raw,
                    &line.content[index..]
                );
            }
        }
    }
}
