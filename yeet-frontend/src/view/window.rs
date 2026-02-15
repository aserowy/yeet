use ratatui::Frame;

use crate::{error::AppError, model::Model};

use super::buffer;

pub fn view(model: &Model, frame: &mut Frame) -> Result<u16, AppError> {
    // NOTE: currently only one window is supported
    buffer::view(&model.state.modes.current, &model.app, frame, 0, 0);

    model.app.window.get_height()
}
