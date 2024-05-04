use std::{collections::HashMap, path::Path};

use yeet_buffer::{message::BufferMessage, update};

use crate::{
    action::Action,
    model::{DirectoryBufferState, Model},
};

use super::{current, cursor, parent, preview};

#[tracing::instrument(skip(model))]
pub fn navigate_to_path(model: &mut Model, path: &Path, selection: &Option<String>) -> Vec<Action> {
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

    let mut current_contents: HashMap<_, _> = HashMap::from([(
        model.files.current.path.clone(),
        model
            .files
            .current
            .buffer
            .lines
            .drain(..)
            .collect::<Vec<_>>(),
    )]);

    if let Some(path) = &model.files.preview.path {
        current_contents.insert(
            path.to_path_buf(),
            model.files.preview.buffer.lines.drain(..).collect(),
        );
    }

    if let Some(path) = &model.files.parent.path {
        current_contents.insert(
            path.to_path_buf(),
            model.files.parent.buffer.lines.drain(..).collect(),
        );
    }

    let mut actions = Vec::new();
    model.files.current.path = path.to_path_buf();
    match current_contents.get(path) {
        Some(it) => {
            // TODO: check if set content and update methods can be combined for current, parent and preview
            update::update_buffer(
                &model.mode,
                &mut model.files.current.buffer,
                &BufferMessage::SetContent(it.to_vec()),
            );
            current::update_current(model, None);

            if let Some(selection) = &selection {
                cursor::set_cursor_index(&model.mode, &mut model.files.current.buffer, selection);
            }
        }
        None => {
            tracing::trace!("loading current: {:?}", path);

            model.files.current.state = DirectoryBufferState::Loading;
            model.files.current.buffer.lines.clear();
            current::update_current(model, None);
            actions.push(Action::Load(path.to_path_buf(), selection.clone()));
        }
    }

    model.files.parent.path = path.parent().map(|path| path.to_path_buf());
    if let Some(parent) = &model.files.parent.path.clone() {
        match current_contents.get(parent) {
            Some(it) => {
                update::update_buffer(
                    &model.mode,
                    &mut model.files.parent.buffer,
                    &BufferMessage::SetContent(it.to_vec()),
                );
                parent::update(model, None);
            }
            None => {
                tracing::trace!("loading parent: {:?}", parent);

                model.files.parent.state = DirectoryBufferState::Loading;
                model.files.parent.buffer.lines.clear();
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

    if let Some(preview) = preview {
        model.files.preview.path = Some(preview.to_path_buf());
        match current_contents.get(&preview) {
            Some(it) => {
                update::update_buffer(
                    &model.mode,
                    &mut model.files.preview.buffer,
                    &BufferMessage::SetContent(it.to_vec()),
                );
                preview::validate_preview_viewport(model);
            }
            None => {
                tracing::trace!("loading preview: {:?}", path);

                model.files.preview.buffer.lines.clear();
                model.files.preview.state = DirectoryBufferState::Loading;
                preview::validate_preview_viewport(model);

                let selection = model.history.get_selection(&preview).map(|s| s.to_owned());
                actions.push(Action::Load(preview, selection));
            }
        }
    } else {
        model.files.preview.buffer.lines.clear();
        preview::validate_preview_viewport(model);
    }

    model.history.add(&model.files.current.path);

    actions
}

#[tracing::instrument(skip(model))]
pub fn navigate_to_parent(model: &mut Model) -> Vec<Action> {
    if let Some(path) = model.files.current.path.clone().parent() {
        if model.files.current.path == path {
            return Vec::new();
        }

        let parent = path.parent();

        let mut actions = Vec::new();
        model.files.parent.path = parent.map(|path| path.to_path_buf());
        if let Some(parent) = parent {
            tracing::trace!("loading parent: {:?}", parent);

            model.files.parent.state = DirectoryBufferState::Loading;
            actions.push(Action::Load(
                parent.to_path_buf(),
                path.file_name().map(|it| it.to_string_lossy().to_string()),
            ));
        }

        model.files.preview.path = Some(model.files.current.path.clone());
        update::update_buffer(
            &model.mode,
            &mut model.files.preview.buffer,
            &BufferMessage::SetContent(model.files.current.buffer.lines.drain(..).collect()),
        );
        preview::validate_preview_viewport(model);

        model.files.current.path = path.to_path_buf();
        update::update_buffer(
            &model.mode,
            &mut model.files.current.buffer,
            &BufferMessage::SetContent(model.files.parent.buffer.lines.drain(..).collect()),
        );
        current::update_current(model, None);

        cursor::set_cursor_index_with_history(
            &model.mode,
            &model.history,
            &mut model.files.current.buffer,
            &model.files.current.path,
        );

        model.files.parent.buffer.lines.clear();
        parent::update(model, None);

        actions
    } else {
        Vec::new()
    }
}

#[tracing::instrument(skip(model))]
pub fn navigate_to_selected(model: &mut Model) -> Vec<Action> {
    if let Some(selected) = current::selection(model) {
        if model.files.current.path == selected || !selected.is_dir() {
            return Vec::new();
        }

        let current_content = model.files.current.buffer.lines.drain(..).collect();

        model.files.current.path = selected.to_path_buf();
        update::update_buffer(
            &model.mode,
            &mut model.files.current.buffer,
            &BufferMessage::SetContent(model.files.preview.buffer.lines.drain(..).collect()),
        );
        current::update_current(model, None);

        cursor::set_cursor_index_with_history(
            &model.mode,
            &model.history,
            &mut model.files.current.buffer,
            &model.files.current.path,
        );

        model.files.parent.path = model.files.current.path.parent().map(|p| p.to_path_buf());
        update::update_buffer(
            &model.mode,
            &mut model.files.parent.buffer,
            &BufferMessage::SetContent(current_content),
        );
        parent::update(model, None);

        let mut actions = Vec::new();
        if let Some(path) = preview::set_preview_to_selected(model) {
            tracing::trace!("loading preview: {:?}", path);

            model.files.preview.state = DirectoryBufferState::Loading;
            preview::validate_preview_viewport(model);

            let selection = model.history.get_selection(&path).map(|s| s.to_owned());
            actions.push(Action::Load(path, selection));
        }

        model.history.add(&model.files.current.path);

        actions
    } else {
        Vec::new()
    }
}
