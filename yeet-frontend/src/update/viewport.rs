use ratatui::layout::Rect;
use yeet_buffer::{
    message::{BufferMessage, ViewPortDirection},
    model::viewport::ViewPort,
};

use crate::{
    action::Action,
    model::{FileTreeBufferSection, Model},
};

use super::{history, selection};

pub fn move_viewport(model: &mut Model, direction: &ViewPortDirection) -> Vec<Action> {
    let msg = BufferMessage::MoveViewPort(direction.clone());
    super::update_current(model, &msg);

    let mut actions = Vec::new();
    if let Some(path) = selection::get_current_selected_path(model) {
        let selection =
            history::get_selection_from_history(&model.history, &path).map(|s| s.to_owned());
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
