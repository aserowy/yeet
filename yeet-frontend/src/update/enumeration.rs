use std::path::PathBuf;

use yeet_buffer::{
    message::{BufferMessage, ViewPortDirection},
    model::{ansi::Ansi, viewport::ViewPort, BufferLine, Mode},
};

use crate::{
    action::Action,
    event::ContentKind,
    model::{App, Buffer, DirectoryBuffer, DirectoryBufferState, State},
    update::{
        app,
        cursor::{set_cursor_index_to_selection, set_cursor_index_with_history},
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
    {
        let App {
            buffers, window, ..
        } = app;
        for (buffer_id, buffer) in buffers.iter_mut() {
            if let Buffer::PathReference(referenced_path) = buffer {
                if referenced_path == path {
                    *buffer = Buffer::Directory(DirectoryBuffer {
                        path: path.clone(),
                        ..Default::default()
                    });
                }
            }

            if let Buffer::Directory(buffer) = buffer {
                if buffer.path.as_path() != path {
                    continue;
                }

                let viewport = app::get_viewport_by_buffer_id_mut(window, *buffer_id);
                set_directory_content(state, viewport, buffer, path, contents, selection);
            }
        }
    }

    let (_, current_id, preview_id) = app::directory_buffer_ids(app);
    let current = match app.buffers.get(&current_id) {
        Some(Buffer::Directory(buffer)) => buffer,
        _ => return Vec::new(),
    };
    if current.path.as_path() != path {
        return Vec::new();
    }

    let mut actions = Vec::new();

    let preview_is_empty = matches!(app.buffers.get(&preview_id), Some(Buffer::Empty));
    if preview_is_empty {
        actions.extend(selection::refresh_preview_from_current_selection(
            app,
            &state.history,
            None,
        ));
    }

    actions
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
    {
        let App {
            buffers, window, ..
        } = app;
        for (buffer_id, buffer) in buffers.iter_mut() {
            if let Buffer::PathReference(referenced_path) = buffer {
                if referenced_path == path {
                    *buffer = Buffer::Directory(DirectoryBuffer {
                        path: path.clone(),
                        ..Default::default()
                    });
                }
            }

            let buffer = match buffer {
                Buffer::Directory(it) => it,
                _ => continue,
            };

            if buffer.path.as_path() != path {
                continue;
            }

            let mut viewport = app::get_viewport_by_buffer_id_mut(window, *buffer_id);
            set_directory_content(
                state,
                viewport.as_deref_mut(),
                buffer,
                path,
                contents,
                selection,
            );

            yeet_buffer::update(
                viewport.as_deref_mut(),
                &state.modes.current,
                &mut buffer.buffer,
                std::slice::from_ref(&BufferMessage::SortContent(super::SORT)),
            );

            if let Some(selection) = selection {
                if !set_cursor_index_to_selection(
                    viewport.as_deref_mut(),
                    &state.modes.current,
                    &mut buffer.buffer,
                    selection,
                ) {
                    set_cursor_index_with_history(
                        &state.history,
                        viewport.as_deref_mut(),
                        &state.modes.current,
                        &mut buffer.buffer,
                        path,
                    );
                }
            }

            let message = BufferMessage::MoveViewPort(ViewPortDirection::CenterOnCursor);
            yeet_buffer::update(
                viewport,
                &state.modes.current,
                &mut buffer.buffer,
                std::slice::from_ref(&message),
            );

            buffer.state = DirectoryBufferState::Ready;
            tracing::trace!(
                "finished enumeration for path {:?}, state is now {:?}",
                path,
                buffer.state,
            );
        }
    }

    let (_, current_id, _) = app::directory_buffer_ids(app);
    let is_current_buffer = match app.buffers.get(&current_id) {
        Some(Buffer::Directory(buffer)) => buffer.path.as_path() == path,
        _ => false,
    };

    if is_current_buffer {
        actions.extend(selection::refresh_preview_from_current_selection(
            app,
            &state.history,
            None,
        ));
    }

    actions
}

fn set_directory_content(
    state: &mut State,
    mut viewport: Option<&mut ViewPort>,
    buffer: &mut DirectoryBuffer,
    path: &PathBuf,
    contents: &[(ContentKind, String)],
    selection: &Option<String>,
) {
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
        viewport.as_deref_mut(),
        &state.modes.current,
        &mut buffer.buffer,
        std::slice::from_ref(&message),
    );

    if is_first_changed_event {
        if let Some(selection) = selection {
            if set_cursor_index_to_selection(
                viewport,
                &state.modes.current,
                &mut buffer.buffer,
                selection,
            ) {
                tracing::trace!("setting cursor index from selection: {:?}", selection);
            }
        }
    }

    buffer.state = DirectoryBufferState::PartiallyLoaded;
    tracing::trace!(
        "changed enumeration for path {:?}, state is now {:?}",
        path,
        buffer.state,
    );
}

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

#[cfg(test)]
mod test {
    use yeet_buffer::model::{ansi::Ansi, Cursor, TextBuffer};

    use crate::{
        action::Action,
        model::{App, Buffer, DirectoryBuffer, Window},
    };

    use super::change;

    #[test]
    fn change_loads_preview_when_empty_for_current() {
        let mut app = App::default();
        let current_path = std::env::current_dir().expect("get current dir");
        let selected_file = current_path.join("Cargo.toml");

        let current_buffer = DirectoryBuffer {
            path: current_path.clone(),
            buffer: TextBuffer {
                lines: vec![yeet_buffer::model::BufferLine {
                    content: Ansi::new("Cargo.toml"),
                    ..Default::default()
                }],
                ..Default::default()
            },
            ..Default::default()
        };

        app.buffers.insert(1, Buffer::Directory(current_buffer));
        app.buffers.insert(2, Buffer::Directory(Default::default()));
        app.buffers.insert(3, Buffer::Empty);
        app.window = Window::Directory(Default::default(), Default::default(), Default::default());
        if let Window::Directory(parent, current, preview) = &mut app.window {
            parent.buffer_id = 2;
            current.buffer_id = 1;
            current.cursor = Cursor {
                vertical_index: 0,
                ..Default::default()
            };
            preview.buffer_id = 3;
        }

        let mut state = crate::model::State::default();
        let actions = change(
            &mut state,
            &mut app,
            &current_path,
            &[(crate::event::ContentKind::File, "Cargo.toml".to_string())],
            &None,
        );

        assert!(matches!(
            actions.as_slice(),
            [Action::Load(path, _)] if path == &selected_file
        ));

        let (_, _, preview_id) = crate::update::app::directory_buffer_ids(&app);
        assert!(matches!(
            app.buffers.get(&preview_id),
            Some(Buffer::PathReference(path)) if path == &selected_file
        ));
    }
}
