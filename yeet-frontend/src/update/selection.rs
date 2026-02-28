use std::path::PathBuf;

use yeet_buffer::model::Cursor;

use crate::model;
use crate::update::preview;
use crate::{
    action::Action,
    event::Message,
    model::{history::History, register::Register, App, Buffer, DirectoryBuffer},
};

use super::{app, history};

#[tracing::instrument(skip(app, history))]
pub fn refresh_preview_from_current_selection(
    app: &mut App,
    history: &History,
    previous_selection: Option<PathBuf>,
) -> Vec<Action> {
    let (_, current_id, _) = app::directory_buffer_ids(app);
    let current_vp = app::get_viewport_by_buffer_id(app, current_id);
    let current_selection = match (app.buffers.get(&current_id), current_vp) {
        (Some(Buffer::Directory(buffer)), Some(vp)) => model::get_selected_path(buffer, &vp.cursor),
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
    cursor: &Cursor,
) -> Vec<Action> {
    if let Some(path) = model::get_selected_path(buffer, cursor) {
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
