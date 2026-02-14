use ratatui::Frame;

use crate::{
    error::AppError,
    model::{Buffer, Model, Window},
    update::app,
};

use super::filetreebuffer;

pub fn view(model: &Model, frame: &mut Frame) -> Result<u16, AppError> {
    // NOTE: extract current shown windows with vp/cursor and buffer id
    let single_window = &model.app.window;

    let (parent_vp, current_vp, preview_vp) = match &single_window {
        Window::Horizontal(_, _) => todo!(),
        Window::Directory(parent, current, preview) => (parent, current, preview),
    };

    let (parent_buffer, current_buffer, preview_buffer) = app::directory_buffers(&model.app);
    let preview_view = match preview_buffer {
        Buffer::Directory(buffer) => filetreebuffer::PreviewView::Directory(buffer),
        Buffer::PreviewImage(buffer) => filetreebuffer::PreviewView::Image(buffer),
        Buffer::_Text(_) => filetreebuffer::PreviewView::None,
    };

    match (parent_buffer, current_buffer) {
        (Buffer::Directory(parent), Buffer::Directory(current)) => filetreebuffer::view(
            filetreebuffer::FileTreeView {
                mode: &model.state.modes.current,
                parent_viewport: parent_vp,
                current_viewport: current_vp,
                preview_viewport: preview_vp,
                parent_buffer: parent,
                current_buffer: current,
                preview_buffer: preview_view,
                horizontal_offset: 0,
                vertical_offset: 0,
            },
            frame,
        ),
        (_, _) => todo!(),
    };

    single_window.get_height()
}
