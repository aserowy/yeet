use ratatui::layout::Rect;
use yeet_buffer::{
    message::{BufferMessage, ViewPortDirection},
    model::{viewport::ViewPort, Mode},
};

use crate::{
    action::Action,
    layout::AppLayout,
    model::{history::History, FileTreeBuffer, FileTreeBufferSection},
};

use super::{history, selection};

pub fn move_viewport(
    history: &History,
    layout: &AppLayout,
    mode: &Mode,
    buffer: &mut FileTreeBuffer,
    direction: &ViewPortDirection,
) -> Vec<Action> {
    let msg = BufferMessage::MoveViewPort(direction.clone());
    super::update_current(layout, mode, buffer, &msg);

    let mut actions = Vec::new();
    if let Some(path) = selection::get_current_selected_path(buffer) {
        let selection =
            history::get_selection_from_history(history, &path).map(|s| s.to_owned());
        actions.push(Action::Load(
            FileTreeBufferSection::Preview,
            path,
            selection,
        ));
    }

    actions
}

pub fn set_viewport_dimensions(vp: &mut ViewPort, rect: &Rect) {
    vp.height = usize::from(rect.height);
    vp.width = usize::from(rect.width);
}
