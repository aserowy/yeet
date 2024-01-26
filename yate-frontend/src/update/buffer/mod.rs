use yate_keymap::message::Message;

use crate::model::buffer::{Buffer, BufferLine, Cursor, CursorPosition, ViewPort};

mod cursor;
mod viewport;

pub fn update(model: &mut Buffer, message: &Message) {
    match message {
        Message::ChangeKeySequence(_) => {}
        Message::ChangeMode(_, _) => {}
        Message::ExecuteCommand => todo!(),
        Message::MoveCursor(count, direction) => {
            cursor::update_by_direction(model, count, direction);
            viewport::update_by_cursor(model);
        }
        Message::MoveViewPort(direction) => viewport::update_by_direction(model, direction),
        Message::PassthroughKeys(raw) => update_line(model, raw),
        Message::Refresh => {}
        Message::SelectCurrent => reset_view(&mut model.view_port, &mut model.cursor),
        Message::SelectParent => reset_view(&mut model.view_port, &mut model.cursor),
        Message::Quit => {}
    }
}

pub fn reset_view(view_port: &mut ViewPort, cursor: &mut Option<Cursor>) {
    view_port.horizontal_index = 0;
    view_port.vertical_index = 0;

    if let Some(cursor) = cursor {
        cursor.vertical_index = 0;

        cursor.horizontial_index = match &cursor.horizontial_index {
            CursorPosition::Absolute(_) => CursorPosition::Absolute(0),
            CursorPosition::End => CursorPosition::End,
            CursorPosition::None => CursorPosition::None,
        }
    }
}

fn update_line(model: &mut Buffer, raw: &str) {
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

        cursor.horizontial_index = CursorPosition::Absolute(index + raw.chars().count());

        line.content = format!(
            "{}{}{}",
            &line.content[..index],
            raw,
            &line.content[index..]
        );
    }
}
