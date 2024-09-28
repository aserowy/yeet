use std::{collections::HashMap, path::Path};

use yeet_buffer::{
    message::{BufferMessage, ViewPortDirection},
    model::{Cursor, CursorPosition},
    update::update_buffer,
};

use crate::{
    action::Action,
    model::{Model, PreviewContent, WindowType},
    update::{history::get_selection_from_history, preview},
};

use super::{
    cursor::{set_cursor_index_to_selection, set_cursor_index_with_history},
    history::add_history_entry,
    preview::validate_preview_viewport,
    selection::{self, get_current_selected_path},
    set_viewport_dimensions,
};

#[tracing::instrument(skip(model))]
pub fn navigate_to_mark(char: &char, model: &mut Model) -> Vec<Action> {
    let path = match model.marks.entries.get(char) {
        Some(it) => it.clone(),
        None => return Vec::new(),
    };

    let selection = path
        .file_name()
        .map(|oss| oss.to_string_lossy().to_string());

    let path = match path.parent() {
        Some(parent) => parent,
        None => &path,
    };

    navigate_to_path_with_selection(model, path, &selection)
}

#[tracing::instrument(skip(model))]
pub fn navigate_to_path(model: &mut Model, path: &Path) -> Vec<Action> {
    let (path, selection) = if path.is_file() {
        tracing::warn!("path is a file, not a directory: {:?}", path);
        let selection = path
            .file_name()
            .map(|oss| oss.to_string_lossy().to_string());

        match path.parent() {
            Some(parent) => (parent, selection),
            None => {
                tracing::warn!(
                    "parent from path with file name could not get resolved: {:?}",
                    path
                );
                return Vec::new();
            }
        }
    } else {
        (path, None)
    };

    navigate_to_path_with_selection(model, path, &selection)
}

pub fn navigate_to_path_as_preview(model: &mut Model, path: &Path) -> Vec<Action> {
    let selection = path
        .file_name()
        .map(|oss| oss.to_string_lossy().to_string());

    let path = match path.parent() {
        Some(parent) => parent,
        None => path,
    };

    navigate_to_path_with_selection(model, path, &selection)
}

#[tracing::instrument(skip(model))]
pub fn navigate_to_path_with_selection(
    model: &mut Model,
    path: &Path,
    selection: &Option<String>,
) -> Vec<Action> {
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
        None => {
            tracing::trace!("getting selection from history for path: {:?}", path);
            get_selection_from_history(&model.history, path).map(|history| history.to_owned())
        }
    };

    tracing::trace!("resolved selection: {:?}", selection);

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

    if let PreviewContent::Buffer(dir) = &model.files.preview {
        current_contents.insert(dir.path.to_path_buf(), dir.buffer.lines.drain(..).collect());
    } else {
        tracing::warn!(
            "navigate_to_path_with_selection called without valid preview content for path {:?}",
            path
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
            update_buffer(
                &model.mode,
                &mut model.files.current.buffer,
                &BufferMessage::SetContent(it.to_vec()),
            );
            update_current(model);

            if let Some(selection) = &selection {
                set_cursor_index_to_selection(
                    &model.mode,
                    &mut model.files.current.buffer,
                    selection,
                );
            }
        }
        None => {
            tracing::trace!("loading current: {:?}", path);

            update_current(model);

            actions.push(Action::Load(
                WindowType::Current,
                path.to_path_buf(),
                selection.clone(),
            ));
        }
    }

    model.files.parent.path = path.parent().map(|path| path.to_path_buf());
    if let Some(parent) = &model.files.parent.path.clone() {
        match current_contents.get(parent) {
            Some(it) => {
                update_buffer(
                    &model.mode,
                    &mut model.files.parent.buffer,
                    &BufferMessage::SetContent(it.to_vec()),
                );
                update_parent(model);
            }
            None => {
                tracing::trace!("loading parent: {:?}", parent);

                update_parent(model);

                actions.push(Action::Load(
                    WindowType::Parent,
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
        None => get_current_selected_path(model),
    };

    if let Some(preview) = preview {
        match current_contents.get(&preview) {
            Some(it) => {
                model.files.preview = preview::create_preview_content(&model.mode, &preview, it);
                validate_preview_viewport(model);
            }
            None => {
                tracing::trace!("loading preview: {:?}", path);

                actions.push(Action::Load(
                    WindowType::Preview,
                    preview,
                    get_selection_from_history(&model.history, &preview).map(|s| s.to_owned()),
                ));
            }
        }
    } else {
        model.files.preview.buffer.lines.clear();
        validate_preview_viewport(model);
    }

    add_history_entry(&mut model.history, &model.files.current.path);

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

            actions.push(Action::Load(
                WindowType::Parent,
                parent.to_path_buf(),
                path.file_name().map(|it| it.to_string_lossy().to_string()),
            ));
        }

        model.files.preview.path = Some(model.files.current.path.clone());
        update_buffer(
            &model.mode,
            &mut model.files.preview.buffer,
            &BufferMessage::SetContent(model.files.current.buffer.lines.drain(..).collect()),
        );
        validate_preview_viewport(model);

        model.files.current.path = path.to_path_buf();
        update_buffer(
            &model.mode,
            &mut model.files.current.buffer,
            &BufferMessage::SetContent(model.files.parent.buffer.lines.drain(..).collect()),
        );
        update_current(model);

        set_cursor_index_with_history(
            &model.mode,
            &model.history,
            &mut model.files.current.buffer,
            &model.files.current.path,
        );

        model.files.parent.buffer.lines.clear();
        update_parent(model);

        actions
    } else {
        Vec::new()
    }
}

#[tracing::instrument(skip(model))]
pub fn navigate_to_selected(model: &mut Model) -> Vec<Action> {
    if let Some(selected) = get_current_selected_path(model) {
        if model.files.current.path == selected || !selected.is_dir() {
            return Vec::new();
        }

        let current_content = model.files.current.buffer.lines.drain(..).collect();

        model.files.current.path = selected.to_path_buf();
        update_buffer(
            &model.mode,
            &mut model.files.current.buffer,
            &BufferMessage::SetContent(model.files.preview.buffer.lines.drain(..).collect()),
        );
        update_current(model);

        set_cursor_index_with_history(
            &model.mode,
            &model.history,
            &mut model.files.current.buffer,
            &model.files.current.path,
        );

        model.files.parent.path = model.files.current.path.parent().map(|p| p.to_path_buf());
        update_buffer(
            &model.mode,
            &mut model.files.parent.buffer,
            &BufferMessage::SetContent(current_content),
        );
        update_parent(model);

        let mut actions = Vec::new();
        if let Some(path) = selection::get_current_selected_path(model) {
            tracing::trace!("loading preview: {:?}", path);

            validate_preview_viewport(model);

            let selection = get_selection_from_history(&model.history, &path).map(|s| s.to_owned());
            actions.push(Action::Load(WindowType::Preview, path, selection));
        }

        add_history_entry(&mut model.history, &model.files.current.path);

        actions
    } else {
        Vec::new()
    }
}

fn update_parent(model: &mut Model) {
    let buffer = &mut model.files.parent.buffer;
    let layout = &model.layout.parent;

    set_viewport_dimensions(&mut buffer.view_port, layout);

    match &model.files.parent.path {
        Some(_) => {
            let current_filename = match model.files.current.path.file_name() {
                Some(content) => content.to_str(),
                None => None,
            };

            let current_line = match current_filename {
                Some(content) => buffer
                    .lines
                    .iter()
                    .position(|line| line.content.to_stripped_string() == content),
                None => None,
            };

            if let Some(index) = current_line {
                if let Some(cursor) = &mut buffer.cursor {
                    cursor.vertical_index = index;
                } else {
                    buffer.cursor = Some(Cursor {
                        horizontal_index: CursorPosition::None,
                        vertical_index: index,
                        ..Default::default()
                    });
                }

                update_buffer(
                    &model.mode,
                    buffer,
                    &BufferMessage::MoveViewPort(ViewPortDirection::CenterOnCursor),
                );
            }
        }
        None => {
            buffer.cursor = None;
            update_buffer(&model.mode, buffer, &BufferMessage::SetContent(vec![]));
        }
    }
}

fn update_current(model: &mut Model) {
    let buffer = &mut model.files.current.buffer;
    let layout = &model.layout.current;

    set_viewport_dimensions(&mut buffer.view_port, layout);
    update_buffer(&model.mode, buffer, &BufferMessage::ResetCursor);
}
