use std::path::PathBuf;

use yeet_buffer::model::{Cursor, Mode};
use yeet_lua::LuaConfiguration;

use crate::error::AppError;
use crate::model;
use crate::update::{cursor, preview};
use crate::{
    action::Action,
    event::{LogSeverity, Message},
    model::{history::History, register::Register, App, Buffer, Contents, DirectoryBuffer, Window},
};

use super::{app, history};

#[tracing::instrument(skip(app, history))]
pub fn refresh_preview_from_current_selection(
    app: &mut App,
    history: &mut History,
    previous_selection: Option<PathBuf>,
    lua: Option<&LuaConfiguration>,
) -> Result<Vec<Action>, AppError> {
    let (window, contents) = app.current_window_and_contents_mut()?;
    let current_window = window.focused_window_mut();

    refresh_preview_from_selection(history, current_window, contents, previous_selection, lua)
}

pub fn refresh_preview_from_selection(
    history: &mut History,
    window: &mut Window,
    contents: &mut Contents,
    previous_selection: Option<PathBuf>,
    lua: Option<&LuaConfiguration>,
) -> Result<Vec<Action>, AppError> {
    let Window::Directory(_, current_vp, _) = window else {
        return Ok(Vec::new());
    };

    let current_buffer_id = current_vp.buffer_id;
    let current_buffer = contents
        .buffers
        .get(&current_buffer_id)
        .ok_or_else(|| AppError::BufferNotFound(current_buffer_id))?;

    let current_selection = match current_buffer {
        Buffer::Directory(buffer) => model::get_selected_path(buffer, &current_vp.cursor),
        _ => return Ok(Vec::new()),
    };

    if previous_selection.is_some() && previous_selection == current_selection {
        if preview_matches_selection(window, contents, &current_selection) {
            tracing::trace!("skipping preview refresh: selection unchanged");
            return Ok(Vec::new());
        }

        tracing::debug!("selection unchanged but preview buffer does not match; refreshing");
    }

    tracing::debug!("refreshing preview for selection: {:?}", current_selection);

    Ok(set_preview_buffer_for_selection(
        window,
        contents,
        history,
        current_selection,
        lua,
    ))
}

fn preview_matches_selection(
    window: &Window,
    contents: &Contents,
    selection: &Option<PathBuf>,
) -> bool {
    let preview_id = match app::get_focused_directory_viewports(window) {
        Some((_, _, preview_vp)) => preview_vp.buffer_id,
        None => return false,
    };

    let preview_buffer = match contents.buffers.get(&preview_id) {
        Some(buf) => buf,
        None => return false,
    };

    let matches = match (preview_buffer, selection) {
        (Buffer::Empty, None) => true,
        (Buffer::Empty, Some(_)) | (Buffer::PathReference(_), Some(_)) => false,
        (buffer, Some(selected_path)) => buffer
            .resolve_path()
            .map(|p| p == selected_path.as_path())
            .unwrap_or(false),
        (_, None) => false,
    };

    matches
}

pub fn set_preview_buffer_for_selection(
    window: &mut Window,
    contents: &mut Contents,
    history: &mut History,
    path_to_preview: Option<PathBuf>,
    lua: Option<&LuaConfiguration>,
) -> Vec<Action> {
    let mut actions = Vec::new();
    let preview_id = if let Some(path_to_preview) = path_to_preview {
        let selection = history::selection(history, &path_to_preview).map(|s| s.to_owned());
        let (id, load) = app::resolve_buffer(contents, &path_to_preview, &selection);
        actions.extend(load);

        history::add_history_entry(history, &path_to_preview);

        id
    } else {
        app::get_empty_buffer(contents)
    };

    preview::set_buffer_id(contents, window, preview_id, lua);

    if let Some((_, _, preview_vp)) = app::get_focused_directory_viewports_mut(window) {
        preview_vp.cursor = Cursor::default();
        preview_vp.hide_cursor_line = true;
        preview_vp.horizontal_index = 0;
        preview_vp.vertical_index = 0;

        let mut cursor_vp = preview_vp.clone();
        let _ = cursor::set_index(contents, history, &mut cursor_vp, &Mode::Normal, None);
        *preview_vp = cursor_vp;
    }

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
                Err(err) => vec![Action::EmitMessages(vec![Message::Log(
                    LogSeverity::Error,
                    err.to_string(),
                )])],
            }
        } else {
            vec![Action::EmitMessages(vec![Message::Log(
                LogSeverity::Error,
                "Clipboard not available".to_string(),
            )])]
        }
    } else {
        Vec::new()
    }
}
