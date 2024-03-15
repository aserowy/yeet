use std::{collections::HashMap, path::Path};

use yeet_keymap::message::Mode;

use crate::{action::Action, model::Model};

use super::{buffer, current, cursor, model::parent, preview};

#[tracing::instrument(skip(model))]
pub fn path(model: &mut Model, path: &Path, selection: &Option<String>) -> Option<Vec<Action>> {
    if path.is_file() {
        tracing::warn!("path is a file, not a directory: {:?}", path);
        return None;
    }

    let mut actions = Vec::new();
    if !path.exists() {
        tracing::warn!("path does not exist: {:?}", path);
        return None;
    }

    let selection = match selection {
        Some(it) => Some(it.to_owned()),
        None => model
            .history
            .get_selection(path)
            .map(|history| history.to_owned()),
    };

    // TODO: invert to reduce clone
    let mut current_contents: HashMap<_, _> = HashMap::from([(
        model.current.path.clone(),
        model.current.buffer.lines.clone(),
    )]);

    if let Some(path) = &model.preview.path {
        current_contents.insert(path.to_path_buf(), model.preview.buffer.lines.clone());
    }

    if let Some(path) = &model.parent.path {
        current_contents.insert(path.to_path_buf(), model.parent.buffer.lines.clone());
    }

    match current_contents.get(path) {
        Some(it) => {
            buffer::set_content(&model.mode, &mut model.current.buffer, it.to_vec());
            current::update(model, None);

            if let Some(selection) = &selection {
                cursor::set_cursor_index(selection, &mut model.current.buffer);
            }
        }
        None => {
            model.current.buffer.lines.clear();
            current::update(model, None);
            actions.push(Action::WatchPath(path.to_path_buf(), selection.clone()));
        }
    }
    model.current.path = path.to_path_buf();

    let path_parent = path.parent();
    if let Some(parent) = path_parent {
        match current_contents.get(parent) {
            Some(it) => {
                buffer::set_content(&model.mode, &mut model.parent.buffer, it.to_vec());
                parent::update(model, None);
            }
            None => {
                model.parent.buffer.lines.clear();
                parent::update(model, None);

                actions.push(Action::WatchPath(
                    parent.to_path_buf(),
                    path.file_name().map(|nm| nm.to_string_lossy().to_string()),
                ));
            }
        }
    }
    model.parent.path = path_parent.map(|path| path.to_path_buf());

    let path_preview = match selection {
        Some(it) => {
            let selection = path.join(it);
            if selection.exists() {
                Some(selection)
            } else {
                None
            }
        }
        None => current::selection(model),
    };

    if let Some(preview) = path_preview.clone() {
        match current_contents.get(&preview) {
            Some(it) => {
                buffer::set_content(&model.mode, &mut model.preview.buffer, it.to_vec());
                preview::viewport(model);
                model.preview.path = Some(preview.to_path_buf());
            }
            None => {
                if let Some(preview_actions) = preview::selected_path(model, true, true) {
                    actions.extend(preview_actions);
                    preview::viewport(model);
                }
            }
        }
    } else {
        model.preview.buffer.lines.clear();
        preview::viewport(model);
    }

    for cached_path in current_contents.keys() {
        if cached_path != path
            && Some(cached_path.to_path_buf()) != path_preview
            && Some(cached_path.as_path()) != path_parent
        {
            actions.push(Action::UnwatchPath(cached_path.clone()));
        }
    }

    model.history.add(&model.current.path);

    Some(actions)
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
        if let Some(parent_parent) = parent_parent {
            actions.push(Action::WatchPath(
                parent_parent.to_path_buf(),
                parent
                    .file_name()
                    .map(|nm| nm.to_string_lossy().to_string()),
            ));
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

        cursor::set_cursor_index_with_history(
            &model.current.path,
            &model.history,
            &mut model.current.buffer,
        );

        if let Some(preview_actions) = preview::selected_path(model, true, false) {
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

        cursor::set_cursor_index_with_history(
            &model.current.path,
            &model.history,
            &mut model.current.buffer,
        );

        if let Some(preview_actions) = preview::selected_path(model, false, true) {
            actions.extend(preview_actions);
        }
        preview::viewport(model);

        buffer::set_content(&model.mode, &mut model.parent.buffer, current_content);
        parent::update(model, None);

        model.history.add(&model.current.path);

        Some(actions)
    } else {
        None
    }
}
