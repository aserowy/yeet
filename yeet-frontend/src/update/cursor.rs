use std::path::Path;

use yeet_buffer::{
    message::{BufferMessage, CursorDirection, Search},
    model::{Buffer, BufferResult, Mode, SearchDirection},
    update::update_buffer,
};

use crate::{
    action::Action,
    model::{history::History, Model},
};

use super::{
    commandline::update_commandline,
    history::get_selection_from_history,
    preview::{set_preview_to_selected, validate_preview_viewport},
    register::{get_direction_from_search_register, get_register},
    search::search_in_buffers,
    update_current,
};

pub fn get_selected_content_from_buffer(model: &Buffer) -> Option<String> {
    let index = match &model.cursor {
        Some(it) => it.vertical_index,
        None => return None,
    };

    model.lines.get(index).map(|line| line.content.clone())
}

pub fn set_cursor_index_to_selection(mode: &Mode, model: &mut Buffer, selection: &str) -> bool {
    let result = update_buffer(
        mode,
        model,
        &BufferMessage::SetCursorToLineContent(selection.to_string()),
    );

    result.contains(&BufferResult::CursorPositionChanged)
}

pub fn set_cursor_index_with_history(
    mode: &Mode,
    history: &History,
    buffer: &mut Buffer,
    path: &Path,
) -> bool {
    if let Some(history) = get_selection_from_history(history, path) {
        set_cursor_index_to_selection(mode, buffer, history)
    } else {
        false
    }
}

pub fn move_cursor(model: &mut Model, rpt: &usize, mtn: &CursorDirection) -> Vec<Action> {
    let msg = BufferMessage::MoveCursor(*rpt, mtn.clone());
    match &model.mode {
        Mode::Command(_) => update_commandline(model, Some(&msg)),
        Mode::Insert | Mode::Navigation | Mode::Normal => {
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
            if let Some(path) = set_preview_to_selected(model) {
                validate_preview_viewport(model);

                let selection =
                    get_selection_from_history(&model.history, &path).map(|s| s.to_owned());
                actions.push(Action::Load(path, selection));
            }

            actions
        }
    }
}
