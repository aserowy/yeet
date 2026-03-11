use std::{mem, path::Path};

use yeet_buffer::model::{viewport::ViewPort, Cursor, Mode};

use crate::{
    action::Action,
    error::AppError,
    model::{history::History, mark::Marks, App, Buffer},
    update::{app, cursor, selection},
};

use super::history;

#[tracing::instrument(skip(app))]
pub fn mark(app: &mut App, history: &History, marks: &Marks, char: &char) -> Vec<Action> {
    let path = match marks.entries.get(char) {
        Some(it) => it.clone(),
        None => return Vec::new(),
    };

    let selection = path
        .file_name()
        .map(|oss| oss.to_string_lossy().to_string());
    match path.parent() {
        Some(parent) => parent,
        None => &path,
    };

    navigate_to_path_with_selection(history, app, path.as_path(), &selection)
}

#[tracing::instrument(skip(app, history))]
pub fn path(app: &mut App, history: &History, path: &Path) -> Vec<Action> {
    let (path, selection) = if path.is_file() {
        tracing::info!("path is a file, not a directory: {:?}", path);

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

    navigate_to_path_with_selection(history, app, path, &selection)
}

pub fn path_as_preview(app: &mut App, history: &History, path: &Path) -> Vec<Action> {
    let selection = path
        .file_name()
        .map(|oss| oss.to_string_lossy().to_string());

    let path = match path.parent() {
        Some(parent) => parent,
        None => path,
    };

    navigate_to_path_with_selection(history, app, path, &selection)
}

#[tracing::instrument(skip(app, history))]
pub fn navigate_to_path_with_selection(
    history: &History,
    app: &mut App,
    path: &Path,
    selection: &Option<String>,
) -> Vec<Action> {
    if path.is_file() {
        tracing::warn!("path is a file, not a directory: {:?}", path);
        return Vec::new();
    }

    let mut actions = Vec::new();

    let current_selection = match selection {
        Some(it) => Some(it.to_owned()),
        None => {
            tracing::trace!("getting selection from history for path: {:?}", path);
            history::selection(history, path).map(|history| history.to_owned())
        }
    };

    tracing::trace!("resolved selection: {:?}", current_selection);

    let (current_id, load) = app::resolve_buffer(&mut app.contents, path, &current_selection);
    actions.extend(load);

    let parent_id = if let (Some(parent), selection) = (path.parent(), path.file_name()) {
        let selection = selection.map(|selection| selection.to_string_lossy().to_string());
        let (id, load) = app::resolve_buffer(&mut app.contents, parent, &selection);
        actions.extend(load);
        id
    } else {
        app::get_empty_buffer(&mut app.contents)
    };

    let (window, contents) = match app.current_window_and_contents_mut() {
        Ok(it) => it,
        Err(_) => return actions,
    };
    let (parent_vp, current_vp, _) = match app::get_focused_directory_viewports_mut(window) {
        Some(vps) => vps,
        None => return actions,
    };

    current_vp.buffer_id = current_id;
    current_vp.cursor = Cursor::default();

    let _ = cursor::set_index(
        contents,
        history,
        current_vp,
        &Mode::Normal,
        current_selection.as_deref(),
    );

    let parent_selection = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string());

    parent_vp.buffer_id = parent_id;
    parent_vp.cursor = Cursor::default();

    let _ = cursor::set_index(
        contents,
        history,
        parent_vp,
        &Mode::Normal,
        parent_selection.as_deref(),
    );

    let preview_path = current_selection.as_ref().map(|selection| {
        let mut preview_path = path.to_path_buf();
        preview_path.push(selection);
        preview_path
    });

    actions.extend(selection::set_preview_buffer_for_selection(
        window,
        contents,
        history,
        preview_path,
    ));

    tracing::debug!(
        "navigate_to_path_with_selection returning {} actions",
        actions.len()
    );

    actions
}

#[tracing::instrument(skip(app))]
pub fn parent(app: &mut App) -> Result<Vec<Action>, AppError> {
    let window = app.current_window()?;
    let (_, current_id, _) = match app::get_focused_directory_buffer_ids(window) {
        Some(ids) => ids,
        None => return Ok(Vec::new()),
    };
    let current_path = match app.contents.buffers.get(&current_id) {
        Some(Buffer::Directory(it)) => it.path.clone(),
        _ => return Err(AppError::BufferNotFound(current_id)),
    };

    if let Some(path) = current_path.parent() {
        if current_path == path {
            return Ok(Vec::new());
        }

        let (window, contents) = app.current_window_and_contents_mut()?;
        let (parent_vp, current_vp, preview_vp) =
            match app::get_focused_directory_viewports_mut(window) {
                Some(vps) => vps,
                None => {
                    return Err(AppError::InvalidState(
                        "no focused directory viewports found in current window".to_string(),
                    ))
                }
            };
        swap_viewport(parent_vp, preview_vp);
        swap_viewport(current_vp, preview_vp);

        let mut actions = Vec::new();

        let selection = path
            .file_name()
            .map(|oss| oss.to_string_lossy().to_string());

        let parent_id = if let Some(parent) = path.parent() {
            let (id, load) = app::resolve_buffer(contents, parent, &selection);
            actions.extend(load);
            id
        } else {
            app::get_empty_buffer(contents)
        };

        parent_vp.buffer_id = parent_id;
        let directory = match contents.buffers.get_mut(&parent_vp.buffer_id) {
            Some(Buffer::Directory(it)) => it,
            _ => return Ok(actions),
        };

        if let Some(selection) = selection {
            cursor::set_cursor_index_to_selection(parent_vp, &Mode::Normal, directory, &selection);
        }

        Ok(actions)
    } else {
        Ok(Vec::new())
    }
}

#[tracing::instrument(skip(app, history))]
pub fn selected(app: &mut App, history: &mut History) -> Result<Vec<Action>, AppError> {
    let (window, contents) = app.current_window_and_contents_mut()?;
    let (parent_vp, current_vp, preview_vp) = match app::get_focused_directory_viewports_mut(window)
    {
        Some(vps) => vps,
        None => return Ok(Vec::new()),
    };
    let preview_buffer = match contents.buffers.get(&preview_vp.buffer_id) {
        Some(Buffer::Directory(it)) => it,
        _ => return Err(AppError::BufferNotFound(preview_vp.buffer_id)),
    };

    history::add_history_entry(history, preview_buffer.path.as_path());

    swap_viewport(parent_vp, preview_vp);
    swap_viewport(current_vp, parent_vp);

    current_vp.hide_cursor_line = false;

    selection::refresh_preview_from_current_selection(app, history, None)
}

fn swap_viewport(vp1: &mut ViewPort, vp2: &mut ViewPort) {
    mem::swap(&mut vp1.buffer_id, &mut vp2.buffer_id);
    mem::swap(&mut vp1.cursor, &mut vp2.cursor);
    mem::swap(&mut vp1.hide_cursor_line, &mut vp2.hide_cursor_line);
    mem::swap(&mut vp1.horizontal_index, &mut vp2.horizontal_index);
    mem::swap(&mut vp1.vertical_index, &mut vp2.vertical_index);
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::path::Path;

    use yeet_buffer::model::viewport::ViewPort;

    use crate::model::{
        history::History, mark::Marks, App, Buffer, Contents, SplitFocus, TasksBuffer, Window,
    };

    use super::*;

    fn make_tasks_focused_app() -> App {
        let mut buffers = HashMap::new();
        buffers.insert(10, Buffer::Empty);
        buffers.insert(11, Buffer::Empty);
        buffers.insert(12, Buffer::Empty);
        buffers.insert(20, Buffer::Tasks(TasksBuffer::default()));

        let mut tabs = HashMap::new();
        tabs.insert(
            1,
            Window::Horizontal {
                first: Box::new(Window::Directory(
                    ViewPort {
                        buffer_id: 10,
                        hide_cursor: true,
                        ..Default::default()
                    },
                    ViewPort {
                        buffer_id: 11,
                        ..Default::default()
                    },
                    ViewPort {
                        buffer_id: 12,
                        hide_cursor: true,
                        ..Default::default()
                    },
                )),
                second: Box::new(Window::Tasks(ViewPort {
                    buffer_id: 20,
                    ..Default::default()
                })),
                focus: SplitFocus::Second,
            },
        );

        App {
            commandline: Default::default(),
            contents: Contents {
                buffers,
                latest_buffer_id: 20,
            },
            tabs,
            current_tab_id: 1,
        }
    }

    #[test]
    fn parent_noop_when_tasks_focused() {
        let mut app = make_tasks_focused_app();
        let actions = parent(&mut app).expect("parent must succeed");
        assert!(actions.is_empty());
    }

    #[test]
    fn selected_noop_when_tasks_focused() {
        let mut app = make_tasks_focused_app();
        let mut history = History::default();
        let actions = selected(&mut app, &mut history).expect("selected must succeed");
        assert!(actions.is_empty());
    }

    #[test]
    fn navigate_to_path_does_not_panic_when_tasks_focused() {
        let mut app = make_tasks_focused_app();
        let history = History::default();
        let target = Path::new("/nonexistent/test/path");

        // Should not panic; viewports remain unchanged
        let _actions = navigate_to_path_with_selection(&history, &mut app, target, &None);

        // Task viewport buffer_id unchanged
        let window = app.current_window().expect("test requires current tab");
        match window {
            Window::Horizontal { second, .. } => match second.as_ref() {
                Window::Tasks(vp) => assert_eq!(vp.buffer_id, 20),
                _ => panic!("expected Tasks"),
            },
            _ => panic!("expected Horizontal"),
        }
    }

    #[test]
    fn mark_does_not_panic_when_tasks_focused() {
        let mut app = make_tasks_focused_app();
        let history = History::default();
        let mut marks = Marks::default();
        marks
            .entries
            .insert('a', std::path::PathBuf::from("/nonexistent/test/path"));

        // Should not panic; viewports remain unchanged
        let _actions = mark(&mut app, &history, &marks, &'a');

        let window = app.current_window().expect("test requires current tab");
        match window {
            Window::Horizontal { second, .. } => match second.as_ref() {
                Window::Tasks(vp) => assert_eq!(vp.buffer_id, 20),
                _ => panic!("expected Tasks"),
            },
            _ => panic!("expected Horizontal"),
        }
    }
}
