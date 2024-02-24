use std::path::{Path, PathBuf};

use crate::{action::Action, model::Model};

use super::{directory, preview};

pub fn add(model: &mut Model, paths: &[PathBuf]) -> Option<Vec<Action>> {
    directory::add_paths(model, paths);

    let mut actions = Vec::new();
    if let Some(preview_actions) = preview::path(model, true, true) {
        actions.extend(preview_actions);
        model.preview.buffer.lines.clear();
        preview::viewport(model);
    }

    if actions.is_empty() {
        None
    } else {
        Some(actions)
    }
}

pub fn remove(model: &mut Model, path: &Path) -> Option<Vec<Action>> {
    if path.starts_with(&model.register.path) {
        model.register.remove(path);
        None
    } else {
        directory::remove_path(model, path);

        let mut actions = Vec::new();
        if let Some(preview_actions) = preview::path(model, true, true) {
            actions.extend(preview_actions);
            model.preview.buffer.lines.clear();
            preview::viewport(model);
        }

        if actions.is_empty() {
            None
        } else {
            Some(actions)
        }
    }
}
