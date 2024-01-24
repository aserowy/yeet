use ratatui::Frame;

use crate::{layout::AppLayout, model::Model};

use super::buffer;

pub fn view_current(model: &mut Model, frame: &mut Frame, rect: &AppLayout) {
    buffer::view(
        &model.mode,
        &model.current_directory,
        frame,
        rect.current_directory,
    );
}

pub fn view_parent(model: &mut Model, frame: &mut Frame, rect: &AppLayout) {
    buffer::view(
        &model.mode,
        &model.parent_directory,
        frame,
        rect.parent_directory,
    );
}
