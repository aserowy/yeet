use ratatui::layout::Rect;
use yeet_buffer::{
    message::{BufferMessage, ViewPortDirection},
    model::viewport::ViewPort,
};

use crate::{action::Action, model::Model};

use super::{
    history::get_selection_from_history,
    preview::{set_preview_to_selected, validate_preview_viewport},
    update_current,
};

pub fn move_viewport(model: &mut Model, direction: &ViewPortDirection) -> Vec<Action> {
    let msg = BufferMessage::MoveViewPort(direction.clone());
    update_current(model, &msg);

    let mut actions = Vec::new();
    if let Some(path) = set_preview_to_selected(model) {
        validate_preview_viewport(model);

        let selection = get_selection_from_history(&model.history, &path).map(|s| s.to_owned());
        actions.push(Action::Load(path, selection));
    }

    actions
}

pub fn set_viewport_dimensions(vp: &mut ViewPort, rect: &Rect) {
    vp.height = usize::from(rect.height);
    vp.width = usize::from(rect.width);
}
