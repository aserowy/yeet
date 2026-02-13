use message::BufferMessage;
use model::{viewport::ViewPort, BufferResult, Cursor, Mode, TextBuffer};
use ratatui::Frame;

pub mod message;
pub mod model;
mod update;
mod view;

pub fn update(
    viewport: Option<&mut ViewPort>,
    cursor: Option<&mut Cursor>,
    mode: &Mode,
    buffer: &mut TextBuffer,
    messages: &[BufferMessage],
) -> Vec<BufferResult> {
    update::update(viewport, cursor, mode, buffer, messages)
}

pub fn update_viewport_by_cursor(viewport: &mut ViewPort, cursor: &Cursor, buffer: &TextBuffer) {
    update::viewport::update_by_cursor(viewport, cursor, buffer)
}

pub fn update_viewport_by_direction(
    viewport: &mut ViewPort,
    cursor: Option<&mut Cursor>,
    buffer: &TextBuffer,
    direction: &message::ViewPortDirection,
) {
    update::viewport::update_by_direction(viewport, cursor, buffer, direction)
}

pub fn view(
    viewport: &ViewPort,
    cursor: Option<&Cursor>,
    mode: &Mode,
    buffer: &TextBuffer,
    frame: &mut Frame,
    horizontal_offset: u16,
    vertical_offset: u16,
) {
    view::view(
        viewport,
        cursor,
        mode,
        buffer,
        frame,
        horizontal_offset,
        vertical_offset,
    )
}
