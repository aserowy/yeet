use crate::{error::AppError, model::Model, terminal::TerminalWrapper};

mod commandline;
mod filetreebuffer;
mod statusline;
mod window;

pub fn model(terminal: &mut TerminalWrapper, model: &Model) -> Result<(), AppError> {
    terminal.draw(|frame| {
        let vertical_offset = window::view(model, frame).expect("Failed to render window view");

        statusline::view(buffer, frame, main[1]);

        commandline::view(
            &model.app.commandline,
            &model.state.modes.current,
            frame,
            vertical_offset + 1,
        )
        .expect("Failed to render commandline view");
    })
}
