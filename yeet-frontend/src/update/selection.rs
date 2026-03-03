use std::path::PathBuf;

use yeet_buffer::model::{Cursor, Mode};

use crate::model;
use crate::update::{cursor, preview};
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
    let (_, current_vp, preview_vp) = app::directory_viewports_mut(&mut app.window);
    let current_selection = match app.contents.buffers.get(&current_vp.buffer_id) {
        Some(Buffer::Directory(buffer)) => model::get_selected_path(buffer, &current_vp.cursor),
        _ => return Vec::new(),
    };

    if previous_selection.is_some() && previous_selection == current_selection {
        tracing::trace!("skipping preview refresh: selection unchanged");
        return Vec::new();
    }

    preview_vp.cursor = Cursor::default();
    preview_vp.hide_cursor_line = true;
    preview_vp.horizontal_index = 0;
    preview_vp.vertical_index = 0;

    tracing::debug!("refreshing preview for selection: {:?}", current_selection);

    let actions = set_preview_buffer_for_selection(app, history, current_selection);

    let (_, _, preview_vp) = app::directory_viewports_mut(&mut app.window);
    cursor::set_index(&mut app.contents, history, preview_vp, &Mode::Normal, None);

    actions
}

fn set_preview_buffer_for_selection(
    app: &mut App,
    history: &History,
    selection: Option<PathBuf>,
) -> Vec<Action> {
    let mut actions = Vec::new();
    let preview_id = if let Some(selected_path) = selection {
        let selection = history::selection(history, &selected_path).map(|s| s.to_owned());
        let (id, load) = app::resolve_buffer(&mut app.contents, &selected_path, &selection);
        actions.extend(load);

        id
    } else {
        app::get_empty_buffer(&mut app.contents)
    };

    preview::set_buffer_id(&mut app.contents, &mut app.window, preview_id);

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
