use yeet_buffer::{
    message::{BufferMessage, ViewPortDirection},
    model::Mode,
};

use crate::{
    action::Action,
    model::{history::History, App, Buffer, DirectoryPane},
    update::app,
};

use super::{history, selection};

pub fn relocate(
    app: &mut App,
    history: &History,
    mode: &Mode,
    direction: &ViewPortDirection,
) -> Vec<Action> {
    let msg = BufferMessage::MoveViewPort(direction.clone());

    let (vp, buffer) = match app::get_focused_mut(app) {
        (vp, Buffer::Directory(it)) => (vp, it),
        (_vp, Buffer::PreviewImage(_)) => return Vec::new(),
        (_vp, Buffer::_Text(_)) => return Vec::new(),
    };

    yeet_buffer::update(
        Some(vp),
        mode,
        &mut buffer.buffer,
        std::slice::from_ref(&msg),
    );

    let mut actions = Vec::new();
    if let Some(path) = selection::get_current_selected_path(buffer, Some(&buffer.buffer.cursor)) {
        let selection = history::get_selection_from_history(history, &path).map(|s| s.to_owned());
        actions.push(Action::Load(DirectoryPane::Preview, path, selection));
    }

    actions
}
