use yeet_buffer::{
    message::{BufferMessage, TextModification},
    model::{CommandMode, Mode},
};

use crate::{action::Action, model::Model};

use super::{
    commandline::update_commandline_on_modification,
    history::get_selection_from_history,
    preview::{set_preview_to_selected, validate_preview_viewport},
    search::search_in_buffers,
    update_current,
};

pub fn modify_buffer(
    model: &mut Model,
    repeat: &usize,
    modification: &TextModification,
) -> Vec<Action> {
    let msg = BufferMessage::Modification(*repeat, modification.clone());
    match model.mode {
        Mode::Command(CommandMode::Command) | Mode::Command(CommandMode::PrintMultiline) => {
            update_commandline_on_modification(model, repeat, modification)
        }
        Mode::Command(_) => {
            let actions = update_commandline_on_modification(model, repeat, modification);

            let term = model
                .commandline
                .buffer
                .lines
                .last()
                .map(|bl| bl.content.clone());

            search_in_buffers(model, term);

            actions
        }
        Mode::Insert | Mode::Normal => {
            update_current(model, &msg);

            let mut actions = Vec::new();
            if let Some(path) = set_preview_to_selected(model) {
                validate_preview_viewport(model);

                let selection =
                    get_selection_from_history(&model.history, &path).map(|s| s.to_owned());
                actions.push(Action::Load(path, selection));
            }

            actions
        }
        Mode::Navigation => Vec::new(),
    }
}
