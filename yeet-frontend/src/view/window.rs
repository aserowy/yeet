use ratatui::Frame;

use crate::{
    error::AppError,
    model::{Model, Window},
};

use super::filetreebuffer;

pub fn view(model: &Model, frame: &mut Frame) -> Result<u16, AppError> {
    let single_window = &model.app.window;

    match &single_window {
        Window::Horizontal(_, _) => todo!(),
        Window::Directory(_, _, _) => {
            filetreebuffer::view(&model.state.modes.current, &model.app, frame, 0, 0)
        }
    };

    single_window.get_height()
}
