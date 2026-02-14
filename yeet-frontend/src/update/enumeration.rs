use std::path::PathBuf;

use yeet_buffer::{
    message::{BufferMessage, ViewPortDirection},
    model::{ansi::Ansi, BufferLine, Mode},
};

use crate::{
    action::Action,
    event::ContentKind,
    model::{App, Buffer, DirectoryBuffer, DirectoryBufferState, DirectoryPane, State},
    update::{
        app,
        cursor::{set_cursor_index_to_selection, set_cursor_index_with_history},
        history::get_selection_from_history,
        selection,
        sign::{set_sign_if_marked, set_sign_if_qfix},
    },
};

#[tracing::instrument(skip(state, app, contents))]
pub fn change(
    state: &mut State,
    app: &mut App,
    path: &PathBuf,
    contents: &[(ContentKind, String)],
    selection: &Option<String>,
) -> Vec<Action> {
    for buffer in app.buffers.values_mut() {
        if let Buffer::Directory(buffer) = buffer {
            change_directory(state, buffer, path, contents, selection);
        }
    }

    Vec::new()
}

fn change_directory(
    state: &mut State,
    buffer: &mut DirectoryBuffer,
    path: &PathBuf,
    contents: &[(ContentKind, String)],
    selection: &Option<String>,
) {
    if buffer.path.as_path() == path {
        tracing::trace!("enumeration changed for buffer: {:?}", path);

        let is_first_changed_event = buffer.buffer.lines.is_empty();
        let content: Vec<BufferLine> = contents
            .iter()
            .map(|(knd, cntnt)| {
                let mut line = from_enumeration(cntnt, knd);
                set_sign_if_marked(&state.marks, &mut line, &path.join(cntnt));
                set_sign_if_qfix(&state.qfix, &mut line, &path.join(cntnt));

                line
            })
            .collect();

        let message = BufferMessage::SetContent(content);
        yeet_buffer::update(
            None,
            &state.modes.current,
            &mut buffer.buffer,
            std::slice::from_ref(&message),
        );

        if is_first_changed_event {
            if let Some(selection) = selection {
                if set_cursor_index_to_selection(
                    None,
                    &state.modes.current,
                    &mut buffer.buffer,
                    selection,
                ) {
                    tracing::trace!("setting cursor index from selection: {:?}", selection);
                }
            }
        }
    }

    if path == &buffer.path {
        buffer.state = DirectoryBufferState::PartiallyLoaded;
    }

    tracing::trace!(
        "changed enumeration for path {:?} with current directory states: current is {:?}",
        path,
        buffer.state,
    );
}

#[tracing::instrument(skip(state, app, contents))]
pub fn finish(
    state: &mut State,
    app: &mut App,
    path: &PathBuf,
    contents: &[(ContentKind, String)],
    selection: &Option<String>,
) -> Vec<Action> {
    if state.modes.current != Mode::Navigation {
        return Vec::new();
    }

    let mut actions = Vec::new();
    for buffer in app.buffers.values_mut() {
        let buffer = match buffer {
            Buffer::Directory(it) => it,
            _ => continue,
        };

        change_directory(state, buffer, path, contents, selection);

        if buffer.path.as_path() == path {
            let message = BufferMessage::SortContent(super::SORT);
            yeet_buffer::update(
                None,
                &state.modes.current,
                &mut buffer.buffer,
                std::slice::from_ref(&message),
            );

            if let Some(selection) = selection {
                if !set_cursor_index_to_selection(
                    None,
                    &state.modes.current,
                    &mut buffer.buffer,
                    selection,
                ) {
                    set_cursor_index_with_history(
                        &state.history,
                        None,
                        &state.modes.current,
                        &mut buffer.buffer,
                        path,
                    );
                }
            }

            let message = BufferMessage::MoveViewPort(ViewPortDirection::CenterOnCursor);
            yeet_buffer::update(
                None,
                &state.modes.current,
                &mut buffer.buffer,
                std::slice::from_ref(&message),
            );
        }

        if path == &buffer.path {
            buffer.state = DirectoryBufferState::Ready;
        }

        tracing::trace!(
            "finished enumeration for path {:?} with current directory states: current is {:?}",
            path,
            buffer.state,
        );
    }

    let (_, current_id, preview_id) = app::directory_buffer_ids(app);
    let current = match app.buffers.get(&current_id) {
        Some(Buffer::Directory(buffer)) => buffer,
        _ => return actions,
    };
    if current.path.as_path() == path {
        if let Some(selected_path) =
            selection::get_current_selected_path(current, Some(&current.buffer.cursor))
        {
            let preview_path = match app.buffers.get(&preview_id) {
                Some(Buffer::Directory(buffer)) => buffer.resolve_path(),
                Some(Buffer::PreviewImage(buffer)) => buffer.resolve_path(),
                Some(Buffer::_Text(_)) | None => None,
            };

            if preview_path != Some(selected_path.as_path()) {
                let selection =
                    get_selection_from_history(&state.history, path).map(|s| s.to_owned());
                actions.push(Action::Load(
                    DirectoryPane::Preview,
                    selected_path,
                    selection,
                ));
            }
        }
    }

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
