use std::{collections::HashMap, path::Path};

use yeet_keymap::message::Mode;

use crate::{action::Action, model::Model};

use super::{buffer, current, history, model::parent, preview};

pub fn path(model: &mut Model, path: &Path) -> Option<Vec<Action>> {
    let path_current = if path.is_file() { path.parent()? } else { path };

    let mut actions = Vec::new();
    if !path_current.exists() {
        return None;
    }

    // TODO: invert to reduce clone
    let mut current: HashMap<_, _> = HashMap::from([(
        model.current.path.clone(),
        model.current.buffer.lines.clone(),
    )]);

    if let Some(path) = &model.preview.path {
        current.insert(path.to_path_buf(), model.preview.buffer.lines.clone());
    }

    if let Some(path) = &model.parent.path {
        current.insert(path.to_path_buf(), model.parent.buffer.lines.clone());
    }

    match current.get(path_current) {
        Some(it) => {
            buffer::set_content(&model.mode, &mut model.current.buffer, it.to_vec());
            current::update(model, None);

            history::set_cursor_index(
                &model.current.path,
                &model.history,
                &mut model.current.buffer,
            );
        }
        None => {
            model.current.buffer.lines.clear();
            current::update(model, None);
            actions.push(Action::WatchPath(path_current.to_path_buf()));
        }
    }
    model.current.path = path_current.to_path_buf();

    let path_parent = path_current.parent();
    if let Some(parent) = path_parent {
        match current.get(parent) {
            Some(it) => {
                buffer::set_content(&model.mode, &mut model.parent.buffer, it.to_vec());
                parent::update(model, None);
            }
            None => {
                model.parent.buffer.lines.clear();
                parent::update(model, None);
                actions.push(Action::WatchPath(parent.to_path_buf()));
            }
        }
    }
    model.parent.path = path_parent.map(|path| path.to_path_buf());

    let path_preview = current::selection(model);
    if let Some(preview) = path_preview.clone() {
        match current.get(&preview) {
            Some(it) => {
                buffer::set_content(&model.mode, &mut model.preview.buffer, it.to_vec());
                preview::viewport(model);
                model.preview.path = Some(preview.to_path_buf());
            }
            None => {
                if let Some(preview_actions) = preview::path(model, true, true) {
                    actions.extend(preview_actions);
                    model.preview.buffer.lines.clear();
                    preview::viewport(model);
                }
            }
        }
    } else {
        model.preview.buffer.lines.clear();
        preview::viewport(model);
    }

    for path in current.keys() {
        if path != path_current
            && Some(path.to_path_buf()) != path_preview
            && Some(path.as_path()) != path_parent
        {
            actions.push(Action::UnwatchPath(path.clone()));
        }
    }

    model.history.add(&model.current.path);

    Some(actions)
}

pub fn path_as_preview(model: &mut Model, path: &Path) -> Option<Vec<Action>> {
    todo!()
}

pub fn parent(model: &mut Model) -> Option<Vec<Action>> {
    if model.mode != Mode::Navigation {
        None
    } else if let Some(parent) = model.current.path.parent() {
        if model.current.path == parent {
            return None;
        }

        let parent_parent = parent.parent();

        let mut actions = Vec::new();
        if let Some(parent) = parent_parent {
            actions.push(Action::WatchPath(parent.to_path_buf()));
        }

        model.parent.path = parent_parent.map(|path| path.to_path_buf());
        model.current.path = parent.to_path_buf();

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
    } else if let Some(selected) = current::selection(model) {
        if model.current.path == selected || !selected.is_dir() {
            return None;
        }

        let mut actions = Vec::new();
        if let Some(parent) = &model.parent.path {
            actions.push(Action::UnwatchPath(parent.clone()));
        }
        model.parent.path = Some(model.current.path.clone());
        model.current.path = selected.to_path_buf();

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
