use ratatui::Frame;

use crate::{layout::AppLayout, model::Model};

mod buffer;
mod commandline;
mod directory;
mod preview;
mod statusline;

pub fn view(model: &mut Model, frame: &mut Frame, layout: &AppLayout) {
    commandline::view(model, frame, layout.commandline);
    directory::view_current(model, frame, layout);
    directory::view_parent(model, frame, layout);
    preview::view(model, frame, layout.preview);
    statusline::view(model, frame, layout.statusline);
}
