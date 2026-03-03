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
    let (current_vp, current_buffer) = app::get_focused_current_mut(app);
    let current_selection = match current_buffer {
        Buffer::Directory(buffer) => model::get_selected_path(buffer, &current_vp.cursor),
        _ => return Vec::new(),
    };

    if previous_selection.is_some() && previous_selection == current_selection {
        tracing::trace!("skipping preview refresh: selection unchanged");
        return Vec::new();
    }

    tracing::debug!("refreshing preview for selection: {:?}", current_selection);

    set_preview_buffer_for_selection(app, history, current_selection)
}

pub fn set_preview_buffer_for_selection(
    app: &mut App,
    history: &History,
    path_to_preview: Option<PathBuf>,
) -> Vec<Action> {
    let mut actions = Vec::new();
    let preview_id = if let Some(path_to_preview) = path_to_preview {
        let selection = history::selection(history, &path_to_preview).map(|s| s.to_owned());
        let (id, load) = app::resolve_buffer(&mut app.contents, &path_to_preview, &selection);
        actions.extend(load);

        id
    } else {
        app::get_empty_buffer(&mut app.contents)
    };

    preview::set_buffer_id(&mut app.contents, &mut app.window, preview_id);

    let preview_vp = app::get_focused_directory_viewports_mut(&mut app.window).2;
    preview_vp.cursor = Cursor::default();
    preview_vp.hide_cursor_line = true;
    preview_vp.horizontal_index = 0;
    preview_vp.vertical_index = 0;

    cursor::set_index(&mut app.contents, history, preview_vp, &Mode::Normal, None);

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
