use yeet_buffer::model::Mode;

use crate::{action::Action, model::Model};

use super::selection::get_current_selected_path;

pub fn open_selected(model: &Model) -> Vec<Action> {
    if model.mode != Mode::Navigation {
        return Vec::new();
    }

    if let Some(selected) = get_current_selected_path(model) {
        if model.settings.stdout_on_open {
            vec![Action::Quit(Some(selected.to_string_lossy().to_string()))]
        } else {
            vec![Action::Open(selected)]
        }
    } else {
        Vec::new()
    }
}
