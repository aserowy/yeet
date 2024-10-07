use std::path::PathBuf;

use crate::{
    action::Action,
    model::{qfix::QFIX_SIGN_ID, Model},
};

use super::{
    selection::{get_current_selected_bufferline, get_current_selected_path},
    sign,
};

pub fn toggle_selected_to_qfix(model: &mut Model) -> Vec<Action> {
    let selected = get_current_selected_path(model);
    if let Some(selected) = selected {
        if model.qfix.entries.contains(&selected) {
            model.qfix.entries.retain(|p| p != &selected);
            if let Some(bl) = get_current_selected_bufferline(model) {
                sign::unset(bl, QFIX_SIGN_ID);
            }
        } else {
            model.qfix.entries.push(selected);
            if let Some(bl) = get_current_selected_bufferline(model) {
                sign::set(bl, QFIX_SIGN_ID);
            }
        }
    }
    Vec::new()
}

pub fn add(model: &mut Model, paths: Vec<PathBuf>) -> Vec<Action> {
    for path in paths {
        if !model.qfix.entries.contains(&path) {
            sign::set_sign_for_path(model, path.as_path(), QFIX_SIGN_ID);
            model.qfix.entries.push(path);
        };
    }
    Vec::new()
}
