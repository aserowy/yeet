use ratatui::Frame;

use crate::{
    error::AppError,
    model::{Buffer, Model, Window},
};

use super::filetreebuffer;

pub fn view(model: &Model, frame: &mut Frame) -> Result<u16, AppError> {
    // NOTE: extract current shown windows with vp/cursor and buffer id
    let single_window = &model.app.window;

    let vp = match &single_window {
        Window::Horizontal(_, _) => todo!(),
        Window::Content(vp) => vp,
    };

    let buffer = model.app.buffers.get(&vp.buffer_id).expect("buffer");
    match buffer {
        Buffer::FileTree(it) => {
            filetreebuffer::view(&model.state.modes.current, vp, it, frame, 0, 0)
        }
        Buffer::_Text(_) => todo!(),
    };

    single_window.get_height()
}
