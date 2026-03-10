use std::slice;

use yeet_buffer::{
    message::{BufferMessage, CursorDirection, Search},
    model::{viewport::ViewPort, BufferResult, Mode, SearchDirection},
};

use crate::{
    action::Action,
    error::AppError,
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
) -> Result<bool, AppError> {
    let directory = match contents.buffers.get_mut(&viewport.buffer_id) {
        Some(Buffer::Directory(it)) => it,
        _ => {
            return Err(AppError::InvalidState(format!(
                "set_index called on non-directory buffer with buffer_id {}",
                viewport.buffer_id
            )))
        }
    };

    Ok(set_cursor_index_for_directory(
        directory, history, viewport, mode, selection,
    ))
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
) -> Result<Vec<Action>, AppError> {
    if matches!(*mtn, CursorDirection::Search(_)) {
        let term = get_register(&state.register, &'/');
        search::buffers(app.contents.buffers.values_mut().collect(), term);
    }

    let current_window = app.current_window()?;
    let premotion_preview_path =
        app::get_focused_directory_buffer_ids(current_window).and_then(|(_, _, preview_id)| {
            app.contents
                .buffers
                .get(&preview_id)
                .and_then(|b| b.resolve_path())
                .map(|p| p.to_path_buf())
        });

    let (window, contents) = app.current_window_and_contents_mut()?;
    let (viewport, buffer) = match app::get_focused_current_mut(window, contents)? {
        (viewport, Buffer::Directory(buffer)) => (viewport, buffer),
        (_, Buffer::Image(_))
        | (_, Buffer::Content(_))
        | (_, Buffer::PathReference(_))
        | (_, Buffer::Tasks(_))
        | (_, Buffer::Empty) => return Ok(Vec::new()),
    };

    let msg = BufferMessage::MoveCursor(*rpt, mtn.clone());
    if let CursorDirection::Search(drctn) = mtn {
        let current_drctn = match get_direction_from_search_register(&state.register) {
            Some(it) => it,
            None => return Ok(Vec::new()),
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

    Ok(selection::refresh_preview_from_current_selection(
        app,
        &state.history,
        premotion_preview_path,
    ))
}
