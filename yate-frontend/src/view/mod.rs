use ratatui::Frame;

use crate::{layout::AppLayout, model::Model};

mod buffer;
mod commandline;
mod current_directory;
mod parent_directory;
mod preview;
mod statusline;

pub fn view(model: &mut Model, frame: &mut Frame, layout: &AppLayout) {
    commandline::view(model, frame, layout.commandline);
    current_directory::view(model, frame, layout.current_directory);
    parent_directory::view(model, frame, layout.parent_directory);
    preview::view(model, frame, layout.preview);
    statusline::view(model, frame, layout.statusline);
}
