use std::path::PathBuf;

use yeet_buffer::{
    message::{BufferMessage, ViewPortDirection},
    model::{ansi::Ansi, BufferLine, Cursor, CursorPosition, Mode},
    update::update_buffer,
};

use crate::{
    action::Action,
    event::ContentKind,
    model::{BufferType, DirectoryBufferState, Model, WindowType},
    update::{
        cursor::{set_cursor_index_to_selection, set_cursor_index_with_history},
        history::get_selection_from_history,
        selection,
        sign::{set_sign_if_marked, set_sign_if_qfix},
    },
};

#[tracing::instrument(skip(model, contents))]
pub fn update_on_enumeration_change(
    model: &mut Model,
    path: &PathBuf,
    contents: &[(ContentKind, String)],
    selection: &Option<String>,
) -> Vec<Action> {
    // TODO: handle unsaved changes
    let directories = model.files.get_mut_directories();
    if let Some((path, buffer)) = directories.into_iter().find(|(p, _)| p == path) {
        tracing::trace!("enumeration changed for buffer: {:?}", path);

        let is_first_changed_event = buffer.lines.is_empty();
        let content = contents
            .iter()
            .map(|(knd, cntnt)| {
                let mut line = from_enumeration(cntnt, knd);
                set_sign_if_marked(&model.marks, &mut line, &path.join(cntnt));
                set_sign_if_qfix(&model.qfix, &mut line, &path.join(cntnt));

                line
            })
            .collect();

        update_buffer(&model.mode, buffer, &BufferMessage::SetContent(content));

        if is_first_changed_event {
            if let Some(selection) = selection {
                if set_cursor_index_to_selection(&model.mode, buffer, selection) {
                    tracing::trace!("setting cursor index from selection: {:?}", selection);
                }
            }
        }
    }

    if path == &model.files.current.path {
        model.files.current.state = DirectoryBufferState::PartiallyLoaded;
    }

    tracing::trace!(
        "changed enumeration for path {:?} with current directory states: current is {:?}",
        path,
        model.files.current.state,
    );

    Vec::new()
}

#[tracing::instrument(skip(model))]
pub fn update_on_enumeration_finished(
    model: &mut Model,
    path: &PathBuf,
    selection: &Option<String>,
) -> Vec<Action> {
    if model.mode != Mode::Navigation {
        return Vec::new();
    }

    let directories = model.files.get_mut_directories();
    if let Some((_, buffer)) = directories.into_iter().find(|(p, _)| p == path) {
        update_buffer(
            &model.mode,
            buffer,
            &BufferMessage::SortContent(super::SORT),
        );

        if let Some(selection) = selection {
            if buffer.cursor.is_none() {
                buffer.cursor = Some(Cursor {
                    horizontal_index: CursorPosition::None,
                    vertical_index: 0,
                    ..Default::default()
                });
            }

            if !set_cursor_index_to_selection(&model.mode, buffer, selection) {
                set_cursor_index_with_history(&model.mode, &model.history, buffer, path);
            }
        }
    }

    if path == &model.files.current.path {
        model.files.current.state = DirectoryBufferState::Ready;
    }

    tracing::trace!(
        "finished enumeration for path {:?} with current directory states: current is {:?}",
        path,
        model.files.current.state,
    );

    if let BufferType::Text(_, buffer) = &mut model.files.parent {
        update_buffer(
            &model.mode,
            buffer,
            &BufferMessage::MoveViewPort(ViewPortDirection::CenterOnCursor),
        );
    }

    let mut actions = Vec::new();
    if model.files.current.state == DirectoryBufferState::Loading {
        return actions;
    }

    let selected_path = match selection::get_current_selected_path(model) {
        Some(path) => path,
        None => return actions,
    };

    if Some(selected_path.as_path()) == model.files.preview.resolve_path() {
        return actions;
    }

    let selection = get_selection_from_history(&model.history, path).map(|s| s.to_owned());
    actions.push(Action::Load(WindowType::Preview, selected_path, selection));

    actions
}

// TODO: move to ansi before
pub fn from_enumeration(content: &String, kind: &ContentKind) -> BufferLine {
    let content = match kind {
        ContentKind::Directory => format!("\x1b[94m{}\x1b[39m", content),
        _ => content.to_string(),
    };

    BufferLine {
        content: Ansi::new(&content),
        ..Default::default()
    }
}
