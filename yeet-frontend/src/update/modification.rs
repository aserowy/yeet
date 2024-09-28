use yeet_buffer::message::{BufferMessage, TextModification};

use crate::{
    action::Action,
    model::{Model, WindowType},
};

use super::{history, preview, selection};

pub fn modify_buffer(
    model: &mut Model,
    repeat: &usize,
    modification: &TextModification,
) -> Vec<Action> {
    let msg = BufferMessage::Modification(*repeat, modification.clone());
    super::update_current(model, &msg);

    let mut actions = Vec::new();
    if let Some(path) = selection::get_current_selected_path(model) {
        preview::validate_preview_viewport(model);

        let selection =
            history::get_selection_from_history(&model.history, &path).map(|s| s.to_owned());
        actions.push(Action::Load(WindowType::Preview, path, selection));
    }

    actions
}
