use yate_keymap::action::Action;

use crate::model::buffer::{Buffer, Cursor};

mod direction;
mod viewport;

pub fn update(model: &mut Buffer, message: &Action) {
    match message {
        Action::ChangeKeySequence(_) => {}
        Action::ChangeMode(_) => {}
        Action::MoveCursor(direction) => direction::update(model, direction),
        Action::Refresh => {}
        Action::SelectCurrent => {
            if model.cursor.is_some() {
                model.cursor = Some(Cursor::default());
            }
        }
        Action::SelectParent => {
            if model.cursor.is_some() {
                model.cursor = Some(Cursor::default());
            }
        }
        Action::Quit => {}
    }
}