use std::{path::PathBuf, slice};

use yeet_buffer::{
    message::{BufferMessage, ViewPortDirection},
    model::{ansi::Ansi, viewport::ViewPort, BufferLine, Mode},
};
use yeet_lua::LuaConfiguration;

use crate::{
    action::Action,
    error::AppError,
    model::{App, Buffer, DirectoryBuffer, DirectoryBufferState, State},
    theme::Theme,
    update::{
        app, cursor, selection,
        sign::{set_sign_if_marked, set_sign_if_qfix},
    },
};

#[tracing::instrument(skip(state, app, content, theme, lua))]
pub fn change(
    state: &mut State,
    app: &mut App,
    path: &PathBuf,
    content: &[String],
    selection: &Option<String>,
    theme: &Theme,
    lua: Option<&LuaConfiguration>,
) -> Result<Vec<Action>, AppError> {
    let (window, contents) = app.current_window_and_contents_mut()?;
    for (buffer_id, buffer) in contents.buffers.iter_mut() {
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
            set_directory_content(
                state, viewport, buffer, path, content, selection, theme, lua,
            );
        }
    }

    let window = app.current_window()?;
    let (current_id, preview_id) = match app::get_focused_directory_buffer_ids(window) {
        Some((_, current_id, preview_id)) => (current_id, preview_id),
        None => {
            return Err(AppError::InvalidState(
                "focused window must have a focused directory buffer".to_string(),
            ))
        }
    };
    let current = match app.contents.buffers.get(&current_id) {
        Some(Buffer::Directory(buffer)) => buffer,
        _ => return Err(AppError::BufferNotFound(current_id)),
    };

    if current.path.as_path() != path {
        return Ok(Vec::new());
    }

    let mut actions = Vec::new();
    let preview_is_empty = matches!(app.contents.buffers.get(&preview_id), Some(Buffer::Empty));
    if preview_is_empty {
        actions.extend(selection::refresh_preview_from_current_selection(
            app,
            &mut state.history,
            None,
        )?);
    }

    Ok(actions)
}

#[tracing::instrument(skip(state, app, content, theme, lua))]
pub fn finish(
    state: &mut State,
    app: &mut App,
    path: &PathBuf,
    content: &[String],
    selection: &Option<String>,
    theme: &Theme,
    lua: Option<&LuaConfiguration>,
) -> Result<Vec<Action>, AppError> {
    if state.modes.current != Mode::Navigation {
        return Ok(Vec::new());
    }

    let mut actions = Vec::new();
    let (tabs, contents) = (&mut app.tabs, &mut app.contents);
    for window in tabs.values_mut() {
        for (buffer_id, buffer) in contents.buffers.iter_mut() {
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
                content,
                selection,
                theme,
                lua,
            );

            yeet_buffer::update(
                viewport.as_deref_mut(),
                &state.modes.current,
                &mut buffer.buffer,
                slice::from_ref(&BufferMessage::SortContent(super::SORT)),
            );

            if let Some(viewport) = viewport.as_deref_mut() {
                cursor::set_cursor_index_for_directory(
                    buffer,
                    &state.history,
                    viewport,
                    &state.modes.current,
                    selection.as_deref(),
                );
            }

            let message = BufferMessage::MoveViewPort(ViewPortDirection::CenterOnCursor);
            yeet_buffer::update(
                viewport,
                &state.modes.current,
                &mut buffer.buffer,
                slice::from_ref(&message),
            );

            buffer.state = DirectoryBufferState::Ready;
            tracing::trace!(
                "finished enumeration for path {:?}, state is now {:?}",
                path,
                buffer.state,
            );
        }
    }

    let mut refresh_tabs: Vec<usize> = app
        .tabs
        .iter()
        .filter_map(|(tab_id, window)| {
            app::get_focused_directory_buffer_ids(window)
                .and_then(|(_, id, _)| app.contents.buffers.get(&id).map(|buffer| (tab_id, buffer)))
                .and_then(|(tab_id, buffer)| match buffer {
                    Buffer::Directory(buffer) if buffer.path.as_path() == path => Some(*tab_id),
                    _ => None,
                })
        })
        .collect();

    refresh_tabs.sort_unstable();
    refresh_tabs.dedup();

    let original_tab = app.current_tab_id;
    for tab_id in refresh_tabs {
        if app.current_tab_id != tab_id {
            app.current_tab_id = tab_id;
        }

        actions.extend(selection::refresh_preview_from_current_selection(
            app,
            &mut state.history,
            None,
        )?);
    }
    app.current_tab_id = original_tab;

    Ok(actions)
}

fn set_directory_content(
    state: &mut State,
    mut viewport: Option<&mut ViewPort>,
    buffer: &mut DirectoryBuffer,
    path: &PathBuf,
    contents: &[String],
    selection: &Option<String>,
    theme: &Theme,
    lua: Option<&LuaConfiguration>,
) {
    tracing::trace!("enumeration changed for buffer: {:?}", path);

    let is_first_changed_event = buffer.buffer.lines.is_empty();
    let content: Vec<BufferLine> = contents
        .iter()
        .map(|cntnt| {
            let mut line = from_enumeration(cntnt);
            if let Some(lua) = lua {
                let bare_name = cntnt.strip_suffix('/').unwrap_or(cntnt);
                yeet_lua::invoke_on_bufferline_mutate(
                    lua,
                    &mut line,
                    "directory",
                    &path.join(bare_name),
                );
            }
            let bare_name = cntnt.strip_suffix('/').unwrap_or(cntnt);
            set_sign_if_marked(&state.marks, &mut line, &path.join(bare_name), theme);
            set_sign_if_qfix(&state.qfix, &mut line, &path.join(bare_name), theme);

            line
        })
        .collect();

    let message = BufferMessage::SetContent(content);
    yeet_buffer::update(
        viewport.as_deref_mut(),
        &state.modes.current,
        &mut buffer.buffer,
        slice::from_ref(&message),
    );

    if is_first_changed_event {
        if let Some(viewport) = viewport {
            if cursor::set_cursor_index_for_directory(
                buffer,
                &state.history,
                viewport,
                &state.modes.current,
                selection.as_deref(),
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

pub fn from_enumeration(content: &str) -> BufferLine {
    BufferLine {
        content: Ansi::new(content),
        ..Default::default()
    }
}

#[cfg(test)]
mod test {
    use std::{env, fs, time::SystemTime};

    use yeet_buffer::model::{ansi::Ansi, viewport::ViewPort, Cursor, TextBuffer};

    use crate::{
        action::Action,
        model::{App, Buffer, DirectoryBuffer, Window},
        theme::Theme,
        update::app,
    };

    use super::{change, finish};

    fn unique_temp_dir() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("yeet-enum-test-{}", nanos))
    }

    #[test]
    fn change_loads_preview_when_empty_for_current() {
        let mut app = App::default();
        let current_path = env::current_dir().expect("get current dir");
        let selected_file = current_path.join("Cargo.toml");

        let current_buffer = DirectoryBuffer {
            path: current_path.clone(),
            buffer: TextBuffer::from_lines(vec![yeet_buffer::model::BufferLine {
                content: Ansi::new("Cargo.toml"),
                ..Default::default()
            }]),
            ..Default::default()
        };

        app.contents
            .buffers
            .insert(1, Buffer::Directory(current_buffer));
        app.contents
            .buffers
            .insert(2, Buffer::Directory(Default::default()));
        app.contents.buffers.insert(3, Buffer::Empty);
        let window = app.current_window_mut().expect("test requires current tab");
        *window = Window::Directory(Default::default(), Default::default(), Default::default());
        if let Window::Directory(parent, current, preview) = window {
            parent.buffer_id = 2;
            current.buffer_id = 1;
            current.cursor = Cursor {
                vertical_index: 0,
                ..Default::default()
            };
            preview.buffer_id = 3;
        }

        let mut state = crate::model::State::default();
        let theme = Theme::default();
        let actions = change(
            &mut state,
            &mut app,
            &current_path,
            &["Cargo.toml".to_string()],
            &None,
            &theme,
            None,
        )
        .expect("change must succeed");

        assert!(matches!(
            actions.as_slice(),
            [Action::Load(path, _)] if *path == selected_file
        ));

        let window = app.current_window().expect("test requires current tab");
        let (_, _, preview_id) =
            crate::update::app::get_focused_directory_buffer_ids(window).unwrap();
        assert!(matches!(
            app.contents.buffers.get(&preview_id),
            Some(Buffer::PathReference(path)) if path == &selected_file
        ));
    }

    #[test]
    fn finish_refreshes_preview_for_all_tabs_with_matching_path() {
        let base = unique_temp_dir();
        fs::create_dir_all(&base).expect("create base dir");
        let file_name = "file.txt";
        let selected_path = base.join(file_name);
        fs::write(&selected_path, "content").expect("create file");

        let mut app = App::default();
        app.tabs.clear();
        app.contents.buffers.clear();
        app.contents.latest_buffer_id = 0;

        let make_tab = |app: &mut App, tab_id: usize, path: &std::path::PathBuf| {
            let parent_id = app::get_next_buffer_id(&mut app.contents);
            let current_id = app::get_next_buffer_id(&mut app.contents);
            let preview_id = app::get_next_buffer_id(&mut app.contents);

            app.contents.buffers.insert(
                parent_id,
                Buffer::Directory(DirectoryBuffer {
                    path: path.clone(),
                    ..Default::default()
                }),
            );
            app.contents.buffers.insert(
                current_id,
                Buffer::Directory(DirectoryBuffer {
                    path: path.clone(),
                    ..Default::default()
                }),
            );
            app.contents.buffers.insert(preview_id, Buffer::Empty);

            app.tabs.insert(
                tab_id,
                Window::Directory(
                    ViewPort {
                        buffer_id: parent_id,
                        ..Default::default()
                    },
                    ViewPort {
                        buffer_id: current_id,
                        ..Default::default()
                    },
                    ViewPort {
                        buffer_id: preview_id,
                        ..Default::default()
                    },
                ),
            );
        };

        make_tab(&mut app, 1, &base);
        make_tab(&mut app, 2, &base);
        app.current_tab_id = 1;

        let mut state = crate::model::State::default();
        state.modes.current = yeet_buffer::model::Mode::Navigation;
        let theme = Theme::default();

        let _ = finish(
            &mut state,
            &mut app,
            &base,
            &[file_name.to_string()],
            &None,
            &theme,
            None,
        )
        .expect("finish must succeed");

        for tab_id in [1, 2] {
            let window = app.tabs.get(&tab_id).expect("tab exists");
            let (_, _, preview_id) = app::get_focused_directory_buffer_ids(window).unwrap();
            let preview_path = app
                .contents
                .buffers
                .get(&preview_id)
                .and_then(|buffer| buffer.resolve_path())
                .map(|path| path.to_path_buf());
            assert_eq!(preview_path, Some(selected_path.clone()));
        }

        assert_eq!(app.current_tab_id, 1);

        let _ = fs::remove_dir_all(&base);
    }
}
