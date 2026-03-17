use message::BufferMessage;
use model::{viewport::ViewPort, BufferResult, Mode, TextBuffer};
use ratatui::Frame;

pub mod message;
pub mod model;
mod update;
pub mod view;

pub fn update(
    viewport: Option<&mut ViewPort>,
    mode: &Mode,
    buffer: &mut TextBuffer,
    messages: &[BufferMessage],
) -> Vec<BufferResult> {
    update::update(viewport, mode, buffer, messages)
}

pub fn update_viewport_by_cursor(viewport: &mut ViewPort, buffer: &TextBuffer) {
    update::viewport::update_by_cursor(viewport, buffer)
}

pub fn update_viewport_by_direction(
    viewport: &mut ViewPort,
    buffer: &mut TextBuffer,
    direction: &message::ViewPortDirection,
) {
    update::viewport::update_by_direction(viewport, buffer, direction);
}

pub fn update_viewport_by_buffer(viewport: &mut ViewPort, mode: &Mode, buffer: &TextBuffer) {
    update::cursor::set_to_inbound_position(&mut viewport.cursor, buffer, mode);
    update::viewport::update_by_cursor(viewport, buffer);
}

pub fn view(
    viewport: &ViewPort,
    mode: &Mode,
    buffer: &TextBuffer,
    frame: &mut Frame,
    styles: view::RenderStyles,
) {
    view::view(viewport, mode, buffer, frame, styles)
}
