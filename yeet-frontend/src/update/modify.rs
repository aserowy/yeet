use yeet_buffer::{
    message::{BufferMessage, TextModification},
    model::Mode,
};

use crate::{
    action::Action,
    model::{App, Buffer, DirectoryPane, State},
    update::app,
};

use super::{history, selection};

pub fn buffer(
    app: &mut App,
    state: &State,
    mode: &Mode,
    repeat: &usize,
    modification: &TextModification,
) -> Vec<Action> {
    let msg = BufferMessage::Modification(*repeat, modification.clone());
    match app::get_focused_mut(app) {
        (vp, Buffer::Directory(it)) => {
            yeet_buffer::update(Some(vp), mode, &mut it.buffer, std::slice::from_ref(&msg));
        }
        (_vp, Buffer::PreviewImage(_)) => return Vec::new(),
        (_vp, Buffer::_Text(_)) => todo!(),
    }

    let (_, buffer) = app::get_focused_mut(app);
    if let Buffer::Directory(buffer) = buffer {
        if let Some(path) =
            selection::get_current_selected_path(buffer, Some(&buffer.buffer.cursor))
        {
            let selection =
                history::get_selection_from_history(&state.history, &path).map(|s| s.to_owned());
            return vec![Action::Load(DirectoryPane::Preview, path, selection)];
        }
    }

    Vec::new()
}
