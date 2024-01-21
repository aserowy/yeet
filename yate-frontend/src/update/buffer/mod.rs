use yate_keymap::message::Message;

use crate::model::buffer::{Buffer, Cursor};

mod direction;
mod viewport;

pub fn update(model: &mut Buffer, message: &Message) {
    match message {
        Message::ChangeKeySequence(_) => {}
        Message::ChangeMode(_) => {}
        Message::MoveCursor(count, direction) => direction::update(model, count, direction),
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
