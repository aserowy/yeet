use ratatui::Frame;

use crate::{error::AppError, model::Model};

pub fn view(model: &Model, frame: &mut Frame) -> Result<u16, AppError> {
    // NOTE: extract current shown windows with vp/cursor and buffer id
    let single_window = model.app.window.first().expect("must exist!");

    let (vp, cursor, id) = match &single_window {
        crate::model::Window::Horizontal(_, _) => todo!(),
        crate::model::Window::Content(vp, cursor, id) => (vp, cursor, id),
    };

    return single_window.get_height();
    // render buffer with the given type
    // inject vp/cursor if file tree buffer
    // set current sizes in vp?
}
