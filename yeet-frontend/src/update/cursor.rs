use std::path::Path;

use yeet_buffer::{
    message::{BufferMessage, CursorDirection, Search},
    model::{viewport::ViewPort, Buffer, BufferResult, Cursor, Mode, SearchDirection},
    update::update_buffer,
};

use crate::{
    action::Action,
    model::{history::History, BufferType, Model, WindowType},
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
    model: &mut Buffer,
    selection: &str,
) -> bool {
    let result = update_buffer(
        viewport,
        cursor,
        mode,
        model,
        &BufferMessage::SetCursorToLineContent(selection.to_string()),
    );

    result.contains(&BufferResult::CursorPositionChanged)
}

pub fn set_cursor_index_with_history(
    viewport: &mut ViewPort,
    cursor: &mut Option<Cursor>,
    mode: &Mode,
    history: &History,
    buffer: &mut Buffer,
    path: &Path,
) -> bool {
    if let Some(history) = get_selection_from_history(history, path) {
        set_cursor_index_to_selection(viewport, cursor, mode, buffer, history)
    } else {
        false
    }
}

pub fn move_cursor(model: &mut Model, rpt: &usize, mtn: &CursorDirection) -> Vec<Action> {
    let premotion_preview_path = match &model.files.preview {
        BufferType::Image(path, _) | BufferType::Text(path, _) => Some(path.clone()),
        BufferType::None => None,
    };

    let msg = BufferMessage::MoveCursor(*rpt, mtn.clone());
    if let CursorDirection::Search(dr) = mtn {
        let term = get_register(&model.register, &'/');
        search_in_buffers(model, term);

        let current_dr = match get_direction_from_search_register(&model.register) {
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
        update_current(model, &msg);
    } else {
        update_current(model, &msg);
    };

    let mut actions = Vec::new();
    let current_preview_path = selection::get_current_selected_path(model);
    if premotion_preview_path == current_preview_path {
        return actions;
    }

    if let Some(path) = current_preview_path {
        let selection = get_selection_from_history(&model.history, &path).map(|s| s.to_owned());
        actions.push(Action::Load(WindowType::Preview, path, selection));
    }

    actions
}
