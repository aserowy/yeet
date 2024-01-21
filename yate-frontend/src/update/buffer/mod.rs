use yate_keymap::action::Action;

use crate::model::buffer::Buffer;

mod direction;
mod viewport;

pub fn update(model: &mut Buffer, message: &Action) {
    match message {
        Action::ChangeKeySequence(_) => {}
        Action::ChangeMode(_) => {}
        Action::MoveCursor(direction) => direction::update(model, direction),
        Action::Refresh => {}
        Action::SelectCurrent => {
            model.cursor = Default::default();
        }
        Action::SelectParent => {
            model.cursor = Default::default();
        }
        Action::Quit => {}
    }
}
