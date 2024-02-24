use std::path::Path;

use yeet_keymap::message::Mode;

use crate::{action::Action, model::Model};

use super::{buffer, current, history, parent, preview};

pub fn path(model: &mut Model, path: &Path) -> Option<Vec<Action>> {
    // TODO: check in set current to path and extend enumeration request with filename
    let directory = if path.is_file() {
        path.parent().unwrap()
    } else {
        path
    };

    let mut actions = Vec::new();
    if let Some(current_actions) = set_current_to_path(model, directory) {
        actions.extend(current_actions);
    }

    model.current.buffer.lines.clear();
    current::update(model, None);

    model.parent.buffer.lines.clear();
    parent::update(model, None);

    model.preview.buffer.lines.clear();
    preview::viewport(model);

    model.history.add(&model.current.path);

    Some(actions)
}

pub fn parent(model: &mut Model) -> Option<Vec<Action>> {
    if model.mode != Mode::Navigation {
        None
    } else if let Some(mut actions) = set_current_to_parent(model) {
        let current_content = model.current.buffer.lines.clone();

        buffer::set_content(
            &model.mode,
            &mut model.current.buffer,
            model.parent.buffer.lines.clone(),
        );
        current::update(model, None);

        history::set_cursor_index(
            &model.current.path,
            &model.history,
            &mut model.current.buffer,
        );

        if let Some(preview_actions) = preview::path(model, true, false) {
            actions.extend(preview_actions);
        }
        buffer::set_content(&model.mode, &mut model.preview.buffer, current_content);
        preview::viewport(model);

        model.parent.buffer.lines.clear();
        parent::update(model, None);

        Some(actions)
    } else {
        None
    }
}

pub fn selected(model: &mut Model) -> Option<Vec<Action>> {
    if model.mode != Mode::Navigation {
        None
    } else if let Some(mut actions) = set_current_to_selected(model) {
        let current_content = model.current.buffer.lines.clone();

        buffer::set_content(
            &model.mode,
            &mut model.current.buffer,
            model.preview.buffer.lines.clone(),
        );
        current::update(model, None);

        history::set_cursor_index(
            &model.current.path,
            &model.history,
            &mut model.current.buffer,
        );

        if let Some(preview_actions) = preview::path(model, false, true) {
            actions.extend(preview_actions);
        }
        model.preview.buffer.lines.clear();
        preview::viewport(model);

        buffer::set_content(&model.mode, &mut model.parent.buffer, current_content);
        parent::update(model, None);

        model.history.add(&model.current.path);

        Some(actions)
    } else {
        None
    }
}

fn set_current_to_parent(model: &mut Model) -> Option<Vec<Action>> {
    if let Some(parent) = model.current.path.parent() {
        if model.current.path == parent {
            return None;
        }

        let parent_parent = parent.parent();

        let mut actions = Vec::new();
        if let Some(parent) = parent_parent {
            actions.extend(vec![
                Action::SleepBeforeRender,
                Action::WatchPath(parent.to_path_buf()),
            ]);
        }

        model.parent.path = parent_parent.map(|path| path.to_path_buf());
        model.current.path = parent.to_path_buf();

        Some(actions)
    } else {
        None
    }
}

fn set_current_to_path(model: &mut Model, path: &Path) -> Option<Vec<Action>> {
    if path.exists() {
        let directory = if path.is_dir() {
            path.to_path_buf()
        } else {
            return None;
        };

        let mut actions = Vec::new();
        if let Some(parent) = &model.parent.path {
            actions.push(Action::UnwatchPath(parent.clone()));
        }

        let parent_parent = directory.parent();
        if let Some(parent) = parent_parent {
            actions.extend(vec![
                Action::SleepBeforeRender,
                Action::WatchPath(parent.to_path_buf()),
            ]);
        }
        model.parent.path = parent_parent.map(|path| path.to_path_buf());

        actions.extend(vec![
            Action::UnwatchPath(model.current.path.clone()),
            Action::SleepBeforeRender,
            Action::WatchPath(directory.clone()),
        ]);
        model.current.path = directory;

        Some(actions)
    } else {
        None
    }
}

fn set_current_to_selected(model: &mut Model) -> Option<Vec<Action>> {
    if let Some(selected) = current::selection(model) {
        if model.current.path == selected || !selected.is_dir() {
            return None;
        }

        let mut actions = Vec::new();
        if let Some(parent) = &model.parent.path {
            actions.push(Action::UnwatchPath(parent.clone()));
        }
        model.parent.path = Some(model.current.path.clone());
        model.current.path = selected.to_path_buf();

        Some(actions)
    } else {
        None
    }
}
