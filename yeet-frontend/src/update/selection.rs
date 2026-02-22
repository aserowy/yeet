use std::path::PathBuf;

use std::path::Path;

use crate::update::preview;
use crate::{
    action::Action,
    event::Message,
    model::{history::History, register::Register, App, Buffer, DirectoryBuffer},
};

use super::{app, history};

pub fn get_current_selected_path(
    buffer: &DirectoryBuffer,
    cursor: Option<&yeet_buffer::model::Cursor>,
) -> Option<PathBuf> {
    get_current_selected_path_with_exists(buffer, cursor, |path| path.exists())
}

pub fn get_current_selected_path_with_exists(
    buffer: &DirectoryBuffer,
    cursor: Option<&yeet_buffer::model::Cursor>,
    exists: impl Fn(&std::path::Path) -> bool,
) -> Option<PathBuf> {
    get_selected_path_with_base(&buffer.path, &buffer.buffer, cursor, exists)
}

pub fn get_selected_path_with_base(
    base_path: &Path,
    text_buffer: &yeet_buffer::model::TextBuffer,
    cursor: Option<&yeet_buffer::model::Cursor>,
    exists: impl Fn(&std::path::Path) -> bool,
) -> Option<PathBuf> {
    if text_buffer.lines.is_empty() {
        return None;
    }

    let cursor = cursor?;
    let current = &text_buffer.lines.get(cursor.vertical_index)?;
    if current.content.is_empty() {
        return None;
    }

    let target = base_path.join(current.content.to_stripped_string());

    if exists(&target) {
        Some(target)
    } else {
        None
    }
}

#[tracing::instrument(skip(app, history))]
pub fn refresh_preview_from_current_selection(
    app: &mut App,
    history: &History,
    previous_selection: Option<PathBuf>,
) -> Vec<Action> {
    let (_, current_id, _) = app::directory_buffer_ids(app);
    let current_selection = match app.buffers.get(&current_id) {
        Some(Buffer::Directory(buffer)) => {
            get_current_selected_path(buffer, Some(&buffer.buffer.cursor))
        }
        _ => return Vec::new(),
    };

    if previous_selection.is_some() && previous_selection == current_selection {
        tracing::trace!("skipping preview refresh: selection unchanged");
        return Vec::new();
    }

    tracing::debug!("refreshing preview for selection: {:?}", current_selection);
    set_preview_buffer_for_selection(app, history, current_selection)
}

fn set_preview_buffer_for_selection(
    app: &mut App,
    history: &History,
    selection: Option<PathBuf>,
) -> Vec<Action> {
    let mut actions = Vec::new();
    let preview_id = if let Some(selected_path) = selection {
        let selection =
            history::get_selection_from_history(history, &selected_path).map(|s| s.to_owned());

        let (id, load) = app::get_or_create_directory_buffer(app, &selected_path, &selection);
        actions.extend(load);

        id
    } else {
        app::create_empty_buffer(app)
    };

    preview::set_buffer_id(app, preview_id);

    actions
}

pub fn copy_to_clipboard(
    register: &mut Register,
    buffer: &DirectoryBuffer,
    cursor: Option<&yeet_buffer::model::Cursor>,
) -> Vec<Action> {
    if let Some(path) = get_current_selected_path(buffer, cursor) {
        if let Some(clipboard) = register.clipboard.as_mut() {
            match clipboard.set_text(path.to_string_lossy()) {
                Ok(_) => Vec::new(),
                Err(err) => vec![Action::EmitMessages(vec![Message::Error(err.to_string())])],
            }
        } else {
            vec![Action::EmitMessages(vec![Message::Error(
                "Clipboard not available".to_string(),
            )])]
        }
    } else {
        Vec::new()
    }
}
