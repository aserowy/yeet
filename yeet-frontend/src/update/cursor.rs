use std::path::Path;

use yeet_buffer::{
    message::{BufferMessage, SearchDirection},
    model::{Buffer, BufferResult, Mode},
    update::{self},
};

use crate::model::history::History;

pub fn get_selection(model: &Buffer) -> Option<String> {
    let index = match &model.cursor {
        Some(it) => it.vertical_index,
        None => return None,
    };

    model.lines.get(index).map(|line| line.content.clone())
}

pub fn set_cursor_index(
    mode: &Mode,
    search: Option<&SearchDirection>,
    model: &mut Buffer,
    selection: &str,
) -> bool {
    let result = update::update(
        mode,
        search,
        model,
        &BufferMessage::SetCursorToLineContent(selection.to_string()),
    );

    matches!(result, Some(BufferResult::CursorPositionChanged))
}

pub fn set_cursor_index_with_history(
    mode: &Mode,
    history: &History,
    search: Option<&SearchDirection>,
    buffer: &mut Buffer,
    path: &Path,
) -> bool {
    if let Some(history) = history.get_selection(path) {
        set_cursor_index(mode, search, buffer, history)
    } else {
        false
    }
}
