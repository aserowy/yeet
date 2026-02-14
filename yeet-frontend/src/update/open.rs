use yeet_buffer::model::Mode;
use yeet_keymap::message::QuitMode;

use crate::{action::Action, model::Buffer, settings::Settings, update::app};

use super::selection::get_selected_path_with_base;

pub fn selected(settings: &Settings, mode: &Mode, app: &mut crate::model::App) -> Vec<Action> {
    if mode != &Mode::Navigation {
        return Vec::new();
    }

    let (parent, _, _) = app::directory_buffers(app);
    let buffer = match parent {
        Buffer::Directory(it) => it,
        _ => return Vec::new(),
    };

    if let Some(selected) = get_selected_path_with_base(
        buffer.path.as_path(),
        &buffer.buffer,
        Some(&buffer.buffer.cursor),
        |path| path.exists(),
    ) {
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
    }
}
