use ratatui::{prelude::Rect, Frame};

use crate::model::Model;

use super::buffer;

pub fn view_current(model: &mut Model, frame: &mut Frame, rect: Rect) {
    buffer::view(&model.mode, &model.current_directory, frame, rect);
}

pub fn view_parent(model: &mut Model, frame: &mut Frame, rect: Rect) {
    buffer::view(&model.mode, &model.parent_directory, frame, rect);
}
