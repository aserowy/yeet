use message::BufferMessage;
use model::{viewport::ViewPort, BufferResult, Mode, TextBuffer};
use ratatui::Frame;

pub mod message;
pub mod model;
mod update;
mod view;

/// Theme data for buffer rendering. All fields are `ratatui::style::Color`.
/// Cursor mode codes and reset sequences are constants in the view module.
#[derive(Debug, Clone)]
pub struct BufferTheme {
    pub buffer_bg: ratatui::style::Color,
    pub buffer_fg: ratatui::style::Color,
    pub cursor_line_bg: ratatui::style::Color,
    pub search_bg: ratatui::style::Color,
    pub line_nr: ratatui::style::Color,
    pub cur_line_nr: ratatui::style::Color,
    pub border_fg: ratatui::style::Color,
    pub border_bg: ratatui::style::Color,
}

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
    theme: &BufferTheme,
    frame: &mut Frame,
) {
    view::view(viewport, mode, buffer, theme, frame)
}
