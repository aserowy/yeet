use ratatui::Frame;

use crate::{error::AppError, model::Model};

use super::buffer;

pub fn view(model: &Model, frame: &mut Frame) -> Result<u16, AppError> {
    buffer::view(&model.state.modes.current, &model.app, frame, 0, 0);

    let window = model.app.current_window()?;
    Ok(window.get_height())
}
