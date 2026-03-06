use crate::{error::AppError, model::Model, terminal::TerminalWrapper};

mod buffer;
mod commandline;
pub(crate) mod statusline;
mod window;

pub fn model(terminal: &mut TerminalWrapper, model: &Model) -> Result<(), AppError> {
    terminal.draw(|frame| {
        tracing::debug!("Rendering with area: {}", frame.area());

        let vertical_offset = window::view(model, frame).expect("Failed to render window view");

        commandline::view(
            &model.app.commandline,
            &model.state.modes.current,
            frame,
            vertical_offset,
        )
        .expect("Failed to render commandline view");
    })
}
