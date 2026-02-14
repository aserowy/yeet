use std::path::Path;

use yeet_buffer::{
    message::{BufferMessage, CursorDirection, Search},
    model::{viewport::ViewPort, BufferResult, Mode, SearchDirection, TextBuffer},
};

use crate::{
    action::Action,
    model::{history::History, App, Buffer, DirectoryPane, State},
};

use super::{
    history::get_selection_from_history,
    register::{get_direction_from_search_register, get_register},
    search, selection,
};

use crate::update::app;

pub fn set_cursor_index_to_selection(
    viewport: Option<&mut ViewPort>,
    mode: &Mode,
    text_buffer: &mut TextBuffer,
    selection: &str,
) -> bool {
    let message = BufferMessage::SetCursorToLineContent(selection.to_string());
    let result = yeet_buffer::update(viewport, mode, text_buffer, std::slice::from_ref(&message));

    result.contains(&BufferResult::CursorPositionChanged)
}

pub fn set_cursor_index_with_history(
    history: &History,
    viewport: Option<&mut ViewPort>,
    mode: &Mode,
    buffer: &mut TextBuffer,
    path: &Path,
) -> bool {
    if let Some(history) = get_selection_from_history(history, path) {
        set_cursor_index_to_selection(viewport, mode, buffer, history)
    } else {
        false
    }
}

pub fn relocate(
    app: &mut App,
    state: &mut State,
    rpt: &usize,
    mtn: &CursorDirection,
) -> Vec<Action> {
    if matches!(*mtn, CursorDirection::Search(_)) {
        let term = get_register(&state.register, &'/');
        search::search_in_buffers(app.buffers.values_mut().collect(), term);
    }

    let (_parent_id, _current_id, preview_id) = app::directory_buffer_ids(app);
    let preview_path = match app.buffers.get(&preview_id) {
        Some(Buffer::Directory(buffer)) => buffer.resolve_path().map(|p| p.to_path_buf()),
        Some(Buffer::PreviewImage(buffer)) => buffer.resolve_path().map(|p| p.to_path_buf()),
        Some(Buffer::_Text(_)) | None => None,
    };

    let (viewport, buffer) = match app::get_focused_mut(app) {
        (viewport, Buffer::Directory(buffer)) => (viewport, buffer),
        (_, Buffer::PreviewImage(_)) => return Vec::new(),
        (_, Buffer::_Text(_)) => todo!(),
    };
    let premotion_preview_path = preview_path;

    let msg = BufferMessage::MoveCursor(*rpt, mtn.clone());
    if let CursorDirection::Search(drctn) = mtn {
        let current_drctn = match get_direction_from_search_register(&state.register) {
            Some(it) => it,
            None => return Vec::new(),
        };

        let dr = match (drctn, current_drctn) {
            (Search::Next, SearchDirection::Down) => Search::Next,
            (Search::Next, SearchDirection::Up) => Search::Previous,
            (Search::Previous, SearchDirection::Down) => Search::Previous,
            (Search::Previous, SearchDirection::Up) => Search::Next,
        };

        let msg = BufferMessage::MoveCursor(*rpt, CursorDirection::Search(dr.clone()));
        yeet_buffer::update(
            Some(viewport),
            &state.modes.current,
            &mut buffer.buffer,
            std::slice::from_ref(&msg),
        );
    } else {
        yeet_buffer::update(
            Some(viewport),
            &state.modes.current,
            &mut buffer.buffer,
            std::slice::from_ref(&msg),
        );
    };

    let mut actions = Vec::new();
    let current_preview_path =
        selection::get_current_selected_path(buffer, Some(&buffer.buffer.cursor));
    if premotion_preview_path == current_preview_path {
        return actions;
    }

    if let Some(path) = selection::get_current_selected_path(buffer, Some(&buffer.buffer.cursor)) {
        let selection = get_selection_from_history(&state.history, &path).map(|s| s.to_owned());
        actions.push(Action::Load(DirectoryPane::Preview, path, selection));
    }

    actions
}
