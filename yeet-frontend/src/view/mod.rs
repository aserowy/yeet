use ratatui::layout::Rect;

use crate::{
    error::AppError,
    model::{Model, Window},
    terminal::TerminalWrapper,
};

mod buffer;
mod commandline;
mod statusline;
mod window;

pub fn model(terminal: &mut TerminalWrapper, model: &Model) -> Result<(), AppError> {
    terminal.draw(|frame| {
        tracing::debug!("Rendering with area: {}", frame.area());

        let vertical_offset = window::view(model, frame).expect("Failed to render window view");
        let focused_id = match &model.app.window {
            Window::Horizontal(_, _) => todo!(),
            Window::Directory(_, vp, _) => &vp.buffer_id,
        };

        let buffer = match model.app.buffers.get(focused_id) {
            Some(it) => it,
            None => todo!(),
        };

        statusline::view(
            buffer,
            frame,
            Rect {
                x: 0,
                width: frame.area().width,
                y: vertical_offset,
                height: 1,
            },
        );

        commandline::view(
            &model.app.commandline,
            &model.state.modes.current,
            frame,
            vertical_offset + 1,
        )
        .expect("Failed to render commandline view");
    })
}
