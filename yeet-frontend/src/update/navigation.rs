use std::{collections::HashMap, path::Path};

use yeet_buffer::{message::BufferMessage, update};

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

    let mut current_contents: HashMap<_, _> = HashMap::from([(
        model.file_buffer.current.path.clone(),
        model
            .file_buffer
            .current
            .buffer
            .lines
            .drain(..)
            .collect::<Vec<_>>(),
    )]);

    if let Some(path) = &model.file_buffer.preview.path {
        current_contents.insert(
            path.to_path_buf(),
            model.file_buffer.preview.buffer.lines.drain(..).collect(),
        );
    }

    if let Some(path) = &model.file_buffer.parent.path {
        current_contents.insert(
            path.to_path_buf(),
            model.file_buffer.parent.buffer.lines.drain(..).collect(),
        );
    }

    let mut actions = Vec::new();
    model.file_buffer.current.path = path.to_path_buf();
    match current_contents.get(path) {
        Some(it) => {
            // TODO: check if set content and update methods can be combined for current, parent and preview
            update::update(
                &model.mode,
                &model.search,
                &mut model.file_buffer.current.buffer,
                &BufferMessage::SetContent(it.to_vec()),
            );
            current::update(model, None);

            if let Some(selection) = &selection {
                cursor::set_cursor_index(
                    &model.mode,
                    &model.search,
                    &mut model.file_buffer.current.buffer,
                    selection,
                );
            }
        }
        None => {
            tracing::trace!("loading current: {:?}", path);

            model.file_buffer.current.state = DirectoryBufferState::Loading;
            model.file_buffer.current.buffer.lines.clear();
            current::update(model, None);
            actions.push(Action::Load(path.to_path_buf(), selection.clone()));
        }
    }

    model.file_buffer.parent.path = path.parent().map(|path| path.to_path_buf());
    if let Some(parent) = &model.file_buffer.parent.path.clone() {
        match current_contents.get(parent) {
            Some(it) => {
                update::update(
                    &model.mode,
                    &model.search,
                    &mut model.file_buffer.parent.buffer,
                    &BufferMessage::SetContent(it.to_vec()),
                );
                parent::update(model, None);
            }
            None => {
                tracing::trace!("loading parent: {:?}", parent);

                model.file_buffer.parent.state = DirectoryBufferState::Loading;
                model.file_buffer.parent.buffer.lines.clear();
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
        model.file_buffer.preview.path = Some(preview.to_path_buf());
        match current_contents.get(&preview) {
            Some(it) => {
                update::update(
                    &model.mode,
                    &model.search,
                    &mut model.file_buffer.preview.buffer,
                    &BufferMessage::SetContent(it.to_vec()),
                );
                preview::viewport(model);
            }
            None => {
                tracing::trace!("loading preview: {:?}", path);

                model.file_buffer.preview.buffer.lines.clear();
                model.file_buffer.preview.state = DirectoryBufferState::Loading;
                preview::viewport(model);

                let selection = model.history.get_selection(&preview).map(|s| s.to_owned());
                actions.push(Action::Load(preview, selection));
            }
        }
    } else {
        model.file_buffer.preview.buffer.lines.clear();
        preview::viewport(model);
    }

    model.history.add(&model.file_buffer.current.path);

    actions
}

#[tracing::instrument(skip(model))]
pub fn parent(model: &mut Model) -> Vec<Action> {
    if let Some(path) = model.file_buffer.current.path.clone().parent() {
        if model.file_buffer.current.path == path {
            return Vec::new();
        }

        let parent = path.parent();

        let mut actions = Vec::new();
        model.file_buffer.parent.path = parent.map(|path| path.to_path_buf());
        if let Some(parent) = parent {
            tracing::trace!("loading parent: {:?}", parent);

            model.file_buffer.parent.state = DirectoryBufferState::Loading;
            actions.push(Action::Load(
                parent.to_path_buf(),
                path.file_name().map(|it| it.to_string_lossy().to_string()),
            ));
        }

        model.file_buffer.preview.path = Some(model.file_buffer.current.path.clone());
        update::update(
            &model.mode,
            &model.search,
            &mut model.file_buffer.preview.buffer,
            &BufferMessage::SetContent(model.file_buffer.current.buffer.lines.drain(..).collect()),
        );
        preview::viewport(model);

        model.file_buffer.current.path = path.to_path_buf();
        update::update(
            &model.mode,
            &model.search,
            &mut model.file_buffer.current.buffer,
            &BufferMessage::SetContent(model.file_buffer.parent.buffer.lines.drain(..).collect()),
        );
        current::update(model, None);

        cursor::set_cursor_index_with_history(
            &model.mode,
            &model.history,
            &model.search,
            &mut model.file_buffer.current.buffer,
            &model.file_buffer.current.path,
        );

        model.file_buffer.parent.buffer.lines.clear();
        parent::update(model, None);

        actions
    } else {
        Vec::new()
    }
}

#[tracing::instrument(skip(model))]
pub fn selected(model: &mut Model) -> Vec<Action> {
    if let Some(selected) = current::selection(model) {
        if model.file_buffer.current.path == selected || !selected.is_dir() {
            return Vec::new();
        }

        let current_content = model.file_buffer.current.buffer.lines.drain(..).collect();

        model.file_buffer.current.path = selected.to_path_buf();
        update::update(
            &model.mode,
            &model.search,
            &mut model.file_buffer.current.buffer,
            &BufferMessage::SetContent(model.file_buffer.preview.buffer.lines.drain(..).collect()),
        );
        current::update(model, None);

        cursor::set_cursor_index_with_history(
            &model.mode,
            &model.history,
            &model.search,
            &mut model.file_buffer.current.buffer,
            &model.file_buffer.current.path,
        );

        model.file_buffer.parent.path = model
            .file_buffer
            .current
            .path
            .parent()
            .map(|p| p.to_path_buf());
        update::update(
            &model.mode,
            &model.search,
            &mut model.file_buffer.parent.buffer,
            &BufferMessage::SetContent(current_content),
        );
        parent::update(model, None);

        let mut actions = Vec::new();
        if let Some(path) = preview::selected_path(model) {
            tracing::trace!("loading preview: {:?}", path);

            model.file_buffer.preview.state = DirectoryBufferState::Loading;
            preview::viewport(model);

            let selection = model.history.get_selection(&path).map(|s| s.to_owned());
            actions.push(Action::Load(path, selection));
        }

        model.history.add(&model.file_buffer.current.path);

        actions
    } else {
        Vec::new()
    }
}
