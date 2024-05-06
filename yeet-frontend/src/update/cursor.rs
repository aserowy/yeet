use std::path::Path;

use yeet_buffer::{
    message::BufferMessage,
    model::{Buffer, BufferResult, Mode},
    update::update_buffer,
};

use crate::model::history::History;

use super::history::get_selection_from_history;

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
