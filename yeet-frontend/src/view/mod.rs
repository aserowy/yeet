use crate::{error::AppError, model::Model, terminal::TerminalWrapper};

mod buffer;
mod commandline;
pub mod statusline;
pub mod tabbar;
mod window;

pub fn model(terminal: &mut TerminalWrapper, model: &Model) -> Result<(), AppError> {
    terminal.draw(|frame| {
        tracing::debug!("Rendering with area: {}", frame.area());

        window::view(model, frame).expect("Failed to render window view");

        commandline::view(&model.app.commandline, &model.state.modes.current, frame)
            .expect("Failed to render commandline view");
    })
}
