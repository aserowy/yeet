use yeet_buffer::message::{BufferMessage, TextModification};

use crate::{action::Action, model::Model};

use super::{
    history::get_selection_from_history,
    preview::{set_preview_to_selected, validate_preview_viewport},
    update_current,
};

pub fn modify_buffer(
    model: &mut Model,
    repeat: &usize,
    modification: &TextModification,
) -> Vec<Action> {
    let msg = BufferMessage::Modification(*repeat, modification.clone());
    update_current(model, &msg);

    let mut actions = Vec::new();
    if let Some(path) = set_preview_to_selected(model) {
        validate_preview_viewport(model);

        let selection = get_selection_from_history(&model.history, &path).map(|s| s.to_owned());
        actions.push(Action::Load(path, selection));
    }

    actions
}
