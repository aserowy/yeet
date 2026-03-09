use std::slice;

use yeet_buffer::{
    message::{BufferMessage, CursorDirection, Search},
    model::{viewport::ViewPort, BufferResult, Mode, SearchDirection},
};

use crate::{
    action::Action,
    model::{history::History, App, Buffer, Contents, DirectoryBuffer, State},
    update::history,
};

use super::{
    register::{get_direction_from_search_register, get_register},
    search, selection,
};

use crate::update::app;

pub fn set_index(
    contents: &mut Contents,
    history: &History,
    viewport: &mut ViewPort,
    mode: &Mode,
    selection: Option<&str>,
) -> bool {
    let directory = match contents.buffers.get_mut(&viewport.buffer_id) {
        Some(Buffer::Directory(it)) => it,
        _ => return false,
    };

    set_cursor_index_for_directory(directory, history, viewport, mode, selection)
}

pub fn set_cursor_index_for_directory(
    directory: &mut DirectoryBuffer,
    history: &History,
    viewport: &mut ViewPort,
    mode: &Mode,
    selection: Option<&str>,
) -> bool {
    if let Some(selection) = selection {
        set_cursor_index_to_selection(viewport, mode, directory, selection)
    } else {
        set_cursor_index_with_history(history, viewport, mode, directory)
    }
}

pub fn set_cursor_index_with_history(
    history: &History,
    viewport: &mut ViewPort,
    mode: &Mode,
    directory: &mut DirectoryBuffer,
) -> bool {
    if let Some(history) = history::selection(history, directory.path.as_path()) {
        set_cursor_index_to_selection(viewport, mode, directory, history)
    } else {
        false
    }
}

pub fn set_cursor_index_to_selection(
    viewport: &mut ViewPort,
    mode: &Mode,
    directory: &mut DirectoryBuffer,
    selection: &str,
) -> bool {
    let message = BufferMessage::SetCursorToLineContent(selection.to_string());
    let result = yeet_buffer::update(
        Some(viewport),
        mode,
        &mut directory.buffer,
        slice::from_ref(&message),
    );

    result.contains(&BufferResult::CursorPositionChanged)
}

pub fn relocate(
    app: &mut App,
    state: &mut State,
    rpt: &usize,
    mtn: &CursorDirection,
) -> Vec<Action> {
    if matches!(*mtn, CursorDirection::Search(_)) {
        let term = get_register(&state.register, &'/');
        search::buffers(app.contents.buffers.values_mut().collect(), term);
    }

    let premotion_preview_path = match app.current_window() {
        Ok(window) => {
            app::get_focused_directory_buffer_ids(window).and_then(|(_, _, preview_id)| {
                app.contents
                    .buffers
                    .get(&preview_id)
                    .and_then(|b| b.resolve_path())
                    .map(|p| p.to_path_buf())
            })
        }
        Err(_) => None,
    };

    let (window, contents) = match app.current_window_and_contents_mut() {
        Ok(window) => window,
        Err(_) => return Vec::new(),
    };
    let (viewport, buffer) = match app::get_focused_current_mut(window, contents) {
        (viewport, Buffer::Directory(buffer)) => (viewport, buffer),
        (_, Buffer::Image(_)) => return Vec::new(),
        (_, Buffer::Content(_)) => return Vec::new(),
        (_, Buffer::PathReference(_)) => return Vec::new(),
        (_, Buffer::Tasks(_)) => return Vec::new(),
        (_, Buffer::Empty) => return Vec::new(),
    };

    let msg = BufferMessage::MoveCursor(*rpt, mtn.clone());
    if let CursorDirection::Search(drctn) = mtn {
        let current_drctn = match get_direction_from_search_register(&state.register) {
            Some(it) => it,
            None => return Vec::new(),
        };

        let direction = match (drctn, current_drctn) {
            (Search::Next, SearchDirection::Down) => Search::Next,
            (Search::Next, SearchDirection::Up) => Search::Previous,
            (Search::Previous, SearchDirection::Down) => Search::Previous,
            (Search::Previous, SearchDirection::Up) => Search::Next,
        };

        let msg = BufferMessage::MoveCursor(*rpt, CursorDirection::Search(direction.clone()));
        yeet_buffer::update(
            Some(viewport),
            &state.modes.current,
            &mut buffer.buffer,
            slice::from_ref(&msg),
        );
    } else {
        yeet_buffer::update(
            Some(viewport),
            &state.modes.current,
            &mut buffer.buffer,
            slice::from_ref(&msg),
        );
    };

    selection::refresh_preview_from_current_selection(app, &state.history, premotion_preview_path)
}
