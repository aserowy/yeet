use message::BufferMessage;
use model::{viewport::ViewPort, BufferResult, Mode, TextBuffer};
use ratatui::Frame;

pub mod message;
pub mod model;
mod update;
mod view;

/// Theme data for buffer rendering. All values are ANSI escape code strings
/// unless noted otherwise.
#[derive(Debug, Clone)]
pub struct BufferTheme {
    pub cursor_line_bg: String,
    pub cursor_line_reset: String,
    pub search_bg: String,
    pub cursor_normal_code: String,
    pub cursor_normal_reset: String,
    pub cursor_insert_code: String,
    pub cursor_insert_reset: String,
    pub line_nr: String,
    pub cur_line_nr_bold: String,
    pub border_fg: String,
    pub border_fg_color: ratatui::style::Color,
    pub border_bg_color: ratatui::style::Color,
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
