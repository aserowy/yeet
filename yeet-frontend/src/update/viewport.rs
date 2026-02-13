use yeet_buffer::{
    message::{BufferMessage, ViewPortDirection},
    model::Mode,
};

use crate::{
    action::Action,
    model::{history::History, App, Buffer, FileTreeBufferSection},
};

use super::{app, history, selection};

pub fn relocate(
    app: &mut App,
    history: &History,
    mode: &Mode,
    direction: &ViewPortDirection,
) -> Vec<Action> {
    let msg = BufferMessage::MoveViewPort(direction.clone());

    let (vp, cursor, buffer) = match app::get_focused_mut(app) {
        (vp, cursor, Buffer::FileTree(it)) => (vp, cursor, it),
        (_vp, _cursor, Buffer::_Text(_)) => return Vec::new(),
    };

    yeet_buffer::update(
        Some(vp),
        Some(cursor),
        mode,
        &mut buffer.current.buffer,
        std::slice::from_ref(&msg),
    );

    let mut actions = Vec::new();
    if let Some(path) = selection::get_current_selected_path(buffer, Some(cursor)) {
        let selection = history::get_selection_from_history(history, &path).map(|s| s.to_owned());
        actions.push(Action::Load(
            FileTreeBufferSection::Preview,
            path,
            selection,
        ));
    }

    actions
}
