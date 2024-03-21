use std::{collections::HashMap, path::Path};

use yeet_buffer::update;

use crate::{
    action::Action,
    model::{DirectoryBufferState, Model},
};

use super::{current, cursor, model::parent, preview};

#[tracing::instrument(skip(model))]
pub fn path(model: &mut Model, path: &Path, selection: &Option<String>) -> Vec<Action> {
    if path.is_file() {
        tracing::warn!("path is a file, not a directory: {:?}", path);
        return Vec::new();
    }

    if !path.exists() {
        tracing::warn!("path does not exist: {:?}", path);
        return Vec::new();
    }

    let selection = match selection {
        Some(it) => Some(it.to_owned()),
        None => model
            .history
            .get_selection(path)
            .map(|history| history.to_owned()),
    };

    // TODO: refactor with model method
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

    let mut actions = Vec::new();
    model.current.path = path.to_path_buf();
    match current_contents.get(path) {
        Some(it) => {
            update::set_content(&model.mode, &mut model.current.buffer, it.to_vec());
            current::update(model, None);

            if let Some(selection) = &selection {
                cursor::set_cursor_index(selection, &mut model.current.buffer);
            }
        }
        None => {
            tracing::trace!("loading current: {:?}", path);

            model.current.state = DirectoryBufferState::Loading;
            model.current.buffer.lines.clear();
            current::update(model, None);
            actions.push(Action::Load(path.to_path_buf(), selection.clone()));
        }
    }

    model.parent.path = path.parent().map(|path| path.to_path_buf());
    if let Some(parent) = &model.parent.path.clone() {
        match current_contents.get(parent) {
            Some(it) => {
                update::set_content(&model.mode, &mut model.parent.buffer, it.to_vec());
                parent::update(model, None);
            }
            None => {
                tracing::trace!("loading parent: {:?}", parent);

                model.parent.state = DirectoryBufferState::Loading;
                model.parent.buffer.lines.clear();
                parent::update(model, None);
                actions.push(Action::Load(
                    parent.to_path_buf(),
                    path.file_name().map(|it| it.to_string_lossy().to_string()),
                ));
            }
        }
    }

    let preview = match selection {
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

    if let Some(preview) = preview.clone() {
        model.preview.path = Some(preview.to_path_buf());
        match current_contents.get(&preview) {
            Some(it) => {
                update::set_content(&model.mode, &mut model.preview.buffer, it.to_vec());
                preview::viewport(model);
            }
            None => {
                tracing::trace!("loading preview: {:?}", path);

                model.preview.buffer.lines.clear();
                model.preview.state = DirectoryBufferState::Loading;
                preview::viewport(model);

                let selection = model.history.get_selection(&preview).map(|s| s.to_owned());
                actions.push(Action::Load(preview, selection));
            }
        }
    } else {
        model.preview.buffer.lines.clear();
        preview::viewport(model);
    }

    model.history.add(&model.current.path);

    actions
}

#[tracing::instrument(skip(model))]
pub fn parent(model: &mut Model) -> Vec<Action> {
    if let Some(path) = model.current.path.clone().parent() {
        if model.current.path == path {
            return Vec::new();
        }

        let parent = path.parent();

        let mut actions = Vec::new();
        model.parent.path = parent.map(|path| path.to_path_buf());
        if let Some(parent) = parent {
            tracing::trace!("loading parent: {:?}", parent);

            model.parent.state = DirectoryBufferState::Loading;
            actions.push(Action::Load(
                parent.to_path_buf(),
                path.file_name().map(|it| it.to_string_lossy().to_string()),
            ));
        }

        model.preview.path = Some(model.current.path.clone());
        update::set_content(
            &model.mode,
            &mut model.preview.buffer,
            model.current.buffer.lines.clone(),
        );
        preview::viewport(model);

        model.current.path = path.to_path_buf();
        update::set_content(
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

        model.parent.buffer.lines.clear();
        parent::update(model, None);

        actions
    } else {
        Vec::new()
    }
}

#[tracing::instrument(skip(model))]
pub fn selected(model: &mut Model) -> Vec<Action> {
    if let Some(selected) = current::selection(model) {
        if model.current.path == selected || !selected.is_dir() {
            return Vec::new();
        }

        let current_content = model.current.buffer.lines.clone();

        model.current.path = selected.to_path_buf();
        update::set_content(
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

        model.parent.path = model.current.path.parent().map(|p| p.to_path_buf());
        update::set_content(&model.mode, &mut model.parent.buffer, current_content);
        parent::update(model, None);

        let mut actions = Vec::new();
        if let Some(path) = preview::selected_path(model) {
            tracing::trace!("loading preview: {:?}", path);

            model.preview.state = DirectoryBufferState::Loading;
            preview::viewport(model);

            let selection = model.history.get_selection(&path).map(|s| s.to_owned());
            actions.push(Action::Load(path, selection));
        }

        model.history.add(&model.current.path);

        actions
    } else {
        Vec::new()
    }
}
