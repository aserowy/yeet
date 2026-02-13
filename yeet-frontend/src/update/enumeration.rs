use std::{mem, path::PathBuf};

use yeet_buffer::{
    message::{BufferMessage, ViewPortDirection},
    model::{ansi::Ansi, BufferLine, Cursor, CursorPosition, Mode},
};

use crate::{
    action::Action,
    event::ContentKind,
    model::{Buffer, DirectoryBufferState, FileTreeBuffer, FileTreeBufferSection, State},
    update::{
        cursor::{set_cursor_index_to_selection, set_cursor_index_with_history},
        history::get_selection_from_history,
        selection,
        sign::{set_sign_if_marked, set_sign_if_qfix},
    },
};

#[tracing::instrument(skip(state, buffers, contents))]
pub fn change(
    state: &mut State,
    buffers: Vec<&mut Buffer>,
    path: &PathBuf,
    contents: &[(ContentKind, String)],
    selection: &Option<String>,
) -> Vec<Action> {
    for buffer in buffers {
        let buffer = match buffer {
            Buffer::FileTree(it) => it,
            _ => continue,
        };

        change_filetree(state, buffer, path, contents, selection);
    }

    Vec::new()
}

fn change_filetree(
    state: &mut State,
    buffer: &mut FileTreeBuffer,
    path: &PathBuf,
    contents: &[(ContentKind, String)],
    selection: &Option<String>,
) {
    let directories = buffer.get_mut_directories();
    if let Some((path, mut cursor, buffer)) = directories.into_iter().find(|(p, _, _)| p == path) {
        tracing::trace!("enumeration changed for buffer: {:?}", path);

        let is_first_changed_event = buffer.lines.is_empty();
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
            cursor.as_deref_mut(),
            &state.modes.current,
            buffer,
            std::slice::from_ref(&message),
        );

        if is_first_changed_event {
            if let Some(selection) = selection {
                if set_cursor_index_to_selection(
                    None,
                    cursor.as_deref_mut(),
                    &state.modes.current,
                    buffer,
                    selection,
                ) {
                    tracing::trace!("setting cursor index from selection: {:?}", selection);
                }
            }
        }
    }

    if path == &buffer.current.path {
        buffer.current.state = DirectoryBufferState::PartiallyLoaded;
    }

    tracing::trace!(
        "changed enumeration for path {:?} with current directory states: current is {:?}",
        path,
        buffer.current.state,
    );
}

#[tracing::instrument(skip(state, buffers, contents))]
pub fn finish(
    state: &mut State,
    buffers: Vec<&mut Buffer>,
    path: &PathBuf,
    contents: &[(ContentKind, String)],
    selection: &Option<String>,
) -> Vec<Action> {
    if state.modes.current != Mode::Navigation {
        return Vec::new();
    }

    let mut actions = Vec::new();
    for buffer in buffers {
        let buffer = match buffer {
            Buffer::FileTree(it) => it,
            _ => continue,
        };

        change_filetree(state, buffer, path, contents, selection);

        let directories = buffer.get_mut_directories();
        if let Some((_, mut cursor, buffer)) = directories.into_iter().find(|(p, _, _)| p == path) {
            let message = BufferMessage::SortContent(super::SORT);
            yeet_buffer::update(
                None,
                cursor.as_deref_mut(),
                &state.modes.current,
                buffer,
                std::slice::from_ref(&message),
            );

            if let Some(selection) = selection {
                let mut cursor_after_finished = match cursor.as_ref() {
                    Some(it) => Some((*it).clone()),
                    None => Some(Cursor {
                        horizontal_index: CursorPosition::None,
                        vertical_index: 0,
                        ..Default::default()
                    }),
                };

                if !set_cursor_index_to_selection(
                    None,
                    cursor_after_finished.as_mut(),
                    &state.modes.current,
                    buffer,
                    selection,
                ) {
                    set_cursor_index_with_history(
                        &state.history,
                        None,
                        cursor_after_finished.as_mut(),
                        &state.modes.current,
                        buffer,
                        path,
                    );
                }

                if let Some(cursor) = cursor.as_deref_mut() {
                    *cursor = cursor_after_finished.unwrap_or_else(|| Cursor {
                        horizontal_index: CursorPosition::None,
                        vertical_index: 0,
                        ..Default::default()
                    });
                }
            }

            let message = BufferMessage::MoveViewPort(ViewPortDirection::CenterOnCursor);
            yeet_buffer::update(
                None,
                cursor.as_deref_mut(),
                &state.modes.current,
                buffer,
                std::slice::from_ref(&message),
            );
        }

        if path == &buffer.current.path {
            buffer.current.state = DirectoryBufferState::Ready;
        }

        tracing::trace!(
            "finished enumeration for path {:?} with current directory states: current is {:?}",
            path,
            buffer.current.state,
        );

        if buffer.current.state == DirectoryBufferState::Loading {
            continue;
        }

        let selected_path =
            match selection::get_current_selected_path(buffer, buffer.parent_cursor.as_ref()) {
                Some(path) => path,
                None => continue,
            };

        if Some(selected_path.as_path()) == buffer.preview.resolve_path() {
            continue;
        }

        let selection = get_selection_from_history(&state.history, path).map(|s| s.to_owned());
        actions.push(Action::Load(
            FileTreeBufferSection::Preview,
            selected_path,
            selection,
        ));
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
