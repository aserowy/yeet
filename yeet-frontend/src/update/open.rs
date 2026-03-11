use yeet_buffer::model::Mode;
use yeet_keymap::message::QuitMode;

use crate::{
    action::Action,
    error::AppError,
    model::{self, App, Buffer},
    settings::Settings,
    update::app,
};

pub fn selected(settings: &Settings, mode: &Mode, app: &mut App) -> Result<Vec<Action>, AppError> {
    if mode != &Mode::Navigation {
        return Ok(Vec::new());
    }

    let (window, contents) = app.current_window_and_contents_mut()?;
    let (current_vp, current_buffer) = app::get_focused_current_mut(window, contents)?;
    let selected = match current_buffer {
        Buffer::Directory(buffer) => model::get_selected_path(buffer, &current_vp.cursor),
        _ => None,
    };

    let result = if let Some(selected) = selected {
        if settings.selection_to_file_on_open.is_some() || settings.selection_to_stdout_on_open {
            vec![Action::Quit(
                QuitMode::FailOnRunningTasks,
                Some(selected.to_string_lossy().to_string()),
            )]
        } else {
            vec![Action::Open(selected)]
        }
    } else {
        Vec::new()
    };

    Ok(result)
}
