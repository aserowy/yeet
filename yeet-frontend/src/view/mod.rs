use yeet_buffer::view;

use crate::{error::AppError, model::Model, terminal::TerminalWrapper};

mod commandline;
mod statusline;

pub fn view(terminal: &mut TerminalWrapper, model: &Model) -> Result<(), AppError> {
    // NOTE: If perf matters, call view only on relevant changed model parts
    terminal.draw(|frame| {
        let layout = model.layout.clone();

        commandline::view(model, frame);

        view::view(&model.mode, &model.current.buffer, frame, layout.current);
        view::view(&model.mode, &model.parent.buffer, frame, layout.parent);
        view::view(&model.mode, &model.preview.buffer, frame, layout.preview);

        statusline::view(model, frame, layout.statusline);
    })
}
