use std::path::Path;

use yeet_buffer::{
    message::{BufferMessage, CursorDirection, Search},
    model::{viewport::ViewPort, BufferResult, Cursor, Mode, SearchDirection, TextBuffer},
    update::update_buffer,
};

use crate::{
    action::Action,
    layout::AppLayout,
    model::{
        history::History, FileTreeBuffer, FileTreeBufferSection,
        FileTreeBufferSectionBuffer, State,
    },
};

use super::{
    history::get_selection_from_history,
    register::{get_direction_from_search_register, get_register},
    search::search_in_buffers,
    selection, update_current,
};

pub fn set_cursor_index_to_selection(
    viewport: &mut ViewPort,
    cursor: &mut Option<Cursor>,
    mode: &Mode,
    text_buffer: &mut TextBuffer,
    selection: &str,
) -> bool {
    let result = update_buffer(
        viewport,
        cursor,
        mode,
        text_buffer,
        &BufferMessage::SetCursorToLineContent(selection.to_string()),
    );

    result.contains(&BufferResult::CursorPositionChanged)
}

pub fn set_cursor_index_with_history(
    history: &History,
    viewport: &mut ViewPort,
    cursor: &mut Option<Cursor>,
    mode: &Mode,
    buffer: &mut TextBuffer,
    path: &Path,
) -> bool {
    if let Some(history) = get_selection_from_history(history, path) {
        set_cursor_index_to_selection(viewport, cursor, mode, buffer, history)
    } else {
        false
    }
}

pub fn move_cursor(
    state: &mut State,
    layout: &AppLayout,
    buffer: &mut FileTreeBuffer,
    rpt: &usize,
    mtn: &CursorDirection,
) -> Vec<Action> {
    let premotion_preview_path = match &buffer.preview {
        FileTreeBufferSectionBuffer::Image(path, _)
        | FileTreeBufferSectionBuffer::Text(path, _) => Some(path.clone()),
        FileTreeBufferSectionBuffer::None => None,
    };

    let msg = BufferMessage::MoveCursor(*rpt, mtn.clone());
    if let CursorDirection::Search(dr) = mtn {
        let term = get_register(&state.register, &'/');
        search_in_buffers(buffer, term);

        let current_dr = match get_direction_from_search_register(&state.register) {
            Some(it) => it,
            None => return Vec::new(),
        };

        let dr = match (dr, current_dr) {
            (Search::Next, SearchDirection::Down) => Search::Next,
            (Search::Next, SearchDirection::Up) => Search::Previous,
            (Search::Previous, SearchDirection::Down) => Search::Previous,
            (Search::Previous, SearchDirection::Up) => Search::Next,
        };

        let msg = BufferMessage::MoveCursor(*rpt, CursorDirection::Search(dr.clone()));
        update_current(layout, &state.modes.current, buffer, &msg);
    } else {
        update_current(layout, &state.modes.current, buffer, &msg);
    };

    let mut actions = Vec::new();
    let current_preview_path = selection::get_current_selected_path(buffer);
    if premotion_preview_path == current_preview_path {
        return actions;
    }

    if let Some(path) = selection::get_current_selected_path(buffer) {
        let selection = get_selection_from_history(&state.history, &path).map(|s| s.to_owned());
        actions.push(Action::Load(
            FileTreeBufferSection::Preview,
            path,
            selection,
        ));
    }

    actions
}
