use ratatui::layout::Rect;

use crate::{error::AppError, model::Model, terminal::TerminalWrapper, update::app};

mod commandline;
mod filetreebuffer;
mod statusline;
mod window;

pub fn model(terminal: &mut TerminalWrapper, model: &Model) -> Result<(), AppError> {
    terminal.draw(|frame| {
        let vertical_offset = window::view(model, frame).expect("Failed to render window view");

        let buffer = app::get_focused(&model.app);

        statusline::view(
            buffer,
            frame,
            Rect {
                x: vertical_offset,
                y: 0,
                width: frame.area().width,
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
