use ratatui::Frame;

use crate::{layout::AppLayout, model::Model};

mod buffer;
mod commandline;
mod preview;
mod statusline;

pub fn view(model: &mut Model, frame: &mut Frame, layout: &AppLayout) {
    // NOTE: If perf matters, call view only on relevant changed model parts
    commandline::view(model, frame, layout.commandline);

    buffer::view(
        &model.mode,
        &model.current.buffer,
        frame,
        layout.current_directory,
    );

    buffer::view(&model.mode, &model.parent, frame, layout.parent_directory);

    preview::view(model, frame, layout.preview);
    statusline::view(model, frame, layout.statusline);
}
