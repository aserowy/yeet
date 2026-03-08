use yeet_buffer::{
    message::{BufferMessage, ViewPortDirection},
    model::Mode,
};

use crate::{
    action::Action,
    model::{history::History, App, Buffer},
    update::app,
};

use super::selection;

pub fn relocate(
    app: &mut App,
    history: &History,
    mode: &Mode,
    direction: &ViewPortDirection,
) -> Vec<Action> {
    let msg = BufferMessage::MoveViewPort(direction.clone());

    let (window, contents) = match app.current_window_and_contents_mut() {
        Ok(window) => window,
        Err(_) => return Vec::new(),
    };
    let (vp, buffer) = match app::get_focused_current_mut(window, contents) {
        (vp, Buffer::Directory(it)) => (vp, it),
        (_vp, Buffer::Image(_)) => return Vec::new(),
        (_vp, Buffer::Content(_)) => return Vec::new(),
        (_vp, Buffer::PathReference(_)) => return Vec::new(),
        (_vp, Buffer::Tasks(_)) => return Vec::new(),
        (_vp, Buffer::Empty) => return Vec::new(),
    };

    yeet_buffer::update(
        Some(vp),
        mode,
        &mut buffer.buffer,
        std::slice::from_ref(&msg),
    );

    selection::refresh_preview_from_current_selection(app, history, None)
}
