use ratatui::Frame;

use crate::{layout::AppLayout, model::Model};

mod buffer;
mod commandline;
mod statusline;

pub fn view(model: &mut Model, frame: &mut Frame, layout: &AppLayout) {
    // NOTE: If perf matters, call view only on relevant changed model parts
    commandline::view(model, frame, layout.commandline);

    buffer::view(&model.mode, &model.current.buffer, frame, layout.current);
    buffer::view(&model.mode, &model.parent.buffer, frame, layout.parent);
    buffer::view(&model.mode, &model.preview.buffer, frame, layout.preview);

    statusline::view(model, frame, layout.statusline);
}
