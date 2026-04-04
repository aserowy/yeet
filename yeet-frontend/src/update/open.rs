use std::path::PathBuf;

use yeet_buffer::model::Mode;
use yeet_keymap::message::{KeymapMessage, QuitMode};

use crate::{
    action::{self, Action},
    error::AppError,
    model::{self, App, Buffer, QuickFixBuffer},
    settings::Settings,
    update::{app, command::qfix::window as qfix_window},
};

pub fn selected(settings: &Settings, mode: &Mode, app: &mut App) -> Result<Vec<Action>, AppError> {
    if mode != &Mode::Navigation {
        return Ok(Vec::new());
    }

    let (window, contents) = app.current_window_and_contents_mut()?;
    let (current_vp, current_buffer) = app::get_focused_current_mut(window, contents)?;

    match current_buffer {
        Buffer::QuickFix(qfix_buf) => {
            let path = get_quickfix_entry_path(qfix_buf, current_vp.cursor.vertical_index);
            if let Some(path) = path {
                let window = app.current_window()?;
                if qfix_window::find_nearest_directory_in_sibling(window).is_some() {
                    return Ok(vec![action::emit_keymap(
                        KeymapMessage::NavigateToPathAsPreview(path),
                    )]);
                }
            }
            Ok(Vec::new())
        }
        Buffer::Directory(buffer) => {
            let selected = model::get_selected_path(buffer, &current_vp.cursor);
            if let Some(selected) = selected {
                if settings.selection_to_file_on_open.is_some()
                    || settings.selection_to_stdout_on_open
                {
                    Ok(vec![Action::Quit(
                        QuitMode::FailOnRunningTasks,
                        Some(selected.to_string_lossy().to_string()),
                    )])
                } else {
                    Ok(vec![Action::Open(selected)])
                }
            } else {
                Ok(Vec::new())
            }
        }
        _ => Ok(Vec::new()),
    }
}

fn get_quickfix_entry_path(qfix_buf: &QuickFixBuffer, cursor_index: usize) -> Option<PathBuf> {
    let line = qfix_buf.buffer.lines.get(cursor_index)?;
    let stripped = line.content.to_stripped_string();
    let path_str = stripped.split_whitespace().nth(1)?;
    Some(PathBuf::from(path_str))
}
