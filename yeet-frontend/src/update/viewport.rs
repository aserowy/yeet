use yeet_buffer::{
    message::{BufferMessage, ViewPortDirection},
    model::Mode,
};

use crate::{
    action::Action,
    error::AppError,
    model::{history::History, App, Buffer},
    update::app,
};

use super::selection;

pub fn relocate(
    app: &mut App,
    history: &History,
    mode: &Mode,
    direction: &ViewPortDirection,
) -> Result<Vec<Action>, AppError> {
    let msg = BufferMessage::MoveViewPort(direction.clone());

    let (window, contents) = app.current_window_and_contents_mut()?;
    let (vp, buffer) = match app::get_focused_current_mut(window, contents)? {
        (vp, Buffer::Directory(it)) => (vp, it),
        (_vp, Buffer::Image(_))
        | (_vp, Buffer::Content(_))
        | (_vp, Buffer::PathReference(_))
        | (_vp, Buffer::Tasks(_))
        | (_vp, Buffer::Empty) => return Ok(Vec::new()),
    };

    yeet_buffer::update(
        Some(vp),
        mode,
        &mut buffer.buffer,
        std::slice::from_ref(&msg),
    );

    selection::refresh_preview_from_current_selection(app, history, None)
}
