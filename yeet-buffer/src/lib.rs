use message::BufferMessage;
use model::{viewport::ViewPort, BufferResult, Mode, TextBuffer};
use ratatui::Frame;

pub mod message;
pub mod model;
mod update;
mod view;

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
    update::viewport::update_by_direction(viewport, buffer, direction)
}

pub fn view(
    viewport: &ViewPort,
    mode: &Mode,
    buffer: &TextBuffer,
    frame: &mut Frame,
    horizontal_offset: u16,
    vertical_offset: u16,
) {
    view::view(
        viewport,
        Some(&buffer.cursor),
        mode,
        buffer,
        frame,
        horizontal_offset,
        vertical_offset,
    )
}
