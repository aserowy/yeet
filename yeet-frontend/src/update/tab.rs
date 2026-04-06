use std::path::{Path, PathBuf};

use yeet_keymap::message::KeymapMessage;

use crate::{
    action::{self, Action},
    error::AppError,
    model::{App, Window},
    update::app,
};

pub fn create_tab(app: &mut App, lua: Option<&yeet_lua::Lua>, target_path: &Path) -> Vec<Action> {
    let empty_buffer = app::get_empty_buffer(&mut app.contents);
    let mut window = Window::create(empty_buffer, empty_buffer, empty_buffer);

    if let Some(lua) = lua {
        super::hook::on_window_create(lua, &mut window, Some(target_path));
    }

    let new_id = next_tab_id(app);
    app.tabs.insert(new_id, window);
    app.current_tab_id = new_id;

    vec![action::emit_keymap(KeymapMessage::NavigateToPath(
        target_path.to_path_buf(),
    ))]
}

pub fn close_tab(app: &mut App) -> Result<Option<usize>, AppError> {
    if app.tabs.len() <= 1 {
        return Ok(None);
    }

    let ordered = ordered_tab_ids(app);
    let Some(next_id) = next_tab_for_close(app.current_tab_id, &ordered) else {
        return Ok(None);
    };

    if app.tabs.remove(&app.current_tab_id).is_none() {
        return Err(AppError::TabNotFound(app.current_tab_id));
    }

    app.current_tab_id = next_id;
    Ok(Some(next_id))
}

pub fn close_other_tabs(app: &mut App) {
    if app.tabs.len() <= 1 {
        return;
    }

    let current = app.current_tab_id;
    app.tabs.retain(|id, _| *id == current);
}

pub fn first_tab(app: &mut App) {
    if let Some(id) = ordered_tab_ids(app).first().copied() {
        app.current_tab_id = id;
    }
}

pub fn last_tab(app: &mut App) {
    if let Some(id) = ordered_tab_ids(app).last().copied() {
        app.current_tab_id = id;
    }
}

pub fn next_tab(app: &mut App) {
    let ordered = ordered_tab_ids(app);
    if let Some(next) = next_tab_id_wrapped(app.current_tab_id, &ordered) {
        app.current_tab_id = next;
    }
}

pub fn previous_tab(app: &mut App) {
    let ordered = ordered_tab_ids(app);
    if let Some(prev) = previous_tab_id_wrapped(app.current_tab_id, &ordered) {
        app.current_tab_id = prev;
    }
}

pub fn ordered_tab_ids(app: &App) -> Vec<usize> {
    let mut ids: Vec<_> = app.tabs.keys().copied().collect();
    ids.sort_unstable();
    ids
}

pub fn tabnew_target_path(app: &App) -> Result<PathBuf, AppError> {
    let current_path = match app.current_window() {
        Ok(window) => match window {
            Window::Directory(_, _, _) => app::get_buffer_path(
                app,
                app::get_focused_directory_buffer_ids(window)
                    .ok_or(AppError::InvalidTargetPath)?
                    .1,
            )
            .ok()
            .flatten(),
            Window::QuickFix(_) | Window::Tasks(_) | Window::Help(_) => None,
            Window::Horizontal { .. } | Window::Vertical { .. } => {
                app::get_focused_directory_buffer_ids(window).and_then(|(_, current_id, _)| {
                    app::get_buffer_path(app, current_id).ok().flatten()
                })
            }
        },
        Err(_) => None,
    };

    if let Some(path) = current_path {
        return Ok(path.to_path_buf());
    }

    dirs::home_dir().ok_or(AppError::InvalidTargetPath)
}

fn next_tab_id(app: &App) -> usize {
    app.tabs.keys().copied().max().unwrap_or(0) + 1
}

fn next_tab_for_close(current: usize, ordered: &[usize]) -> Option<usize> {
    let pos = ordered.iter().position(|id| *id == current)?;
    if pos + 1 < ordered.len() {
        Some(ordered[pos + 1])
    } else if pos > 0 {
        Some(ordered[pos - 1])
    } else {
        None
    }
}

fn next_tab_id_wrapped(current: usize, ordered: &[usize]) -> Option<usize> {
    let pos = ordered.iter().position(|id| *id == current)?;
    if ordered.is_empty() {
        None
    } else {
        Some(ordered[(pos + 1) % ordered.len()])
    }
}

fn previous_tab_id_wrapped(current: usize, ordered: &[usize]) -> Option<usize> {
    let pos = ordered.iter().position(|id| *id == current)?;
    if ordered.is_empty() {
        None
    } else if pos == 0 {
        ordered.last().copied()
    } else {
        Some(ordered[pos - 1])
    }
}
