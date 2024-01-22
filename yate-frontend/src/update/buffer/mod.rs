use yate_keymap::message::Message;

use crate::model::buffer::{Buffer, Cursor};

mod cursor;
mod viewport;

pub fn update(model: &mut Buffer, message: &Message) {
    match message {
        Message::ChangeKeySequence(_) => {}
        Message::ChangeMode(_) => {}
        Message::MoveCursor(count, direction) => {
            cursor::update_by_direction(model, count, direction);
            viewport::update_by_cursor(model);
        }
        Message::MoveViewPort(direction) => viewport::update_by_direction(model, direction),
        Message::Refresh => {}
        Message::SelectCurrent => {
            if model.cursor.is_some() {
                model.cursor = Some(Cursor::default());
            }
        }
        Message::SelectParent => {
            if model.cursor.is_some() {
                model.cursor = Some(Cursor::default());
            }
        }
        Message::Quit => {}
    }
}
