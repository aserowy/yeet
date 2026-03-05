use yeet_buffer::model::Mode;
use yeet_keymap::message::QuitMode;

use crate::{
    action::Action,
    model::{self, Buffer},
    settings::Settings,
    update::app,
};

pub fn selected(settings: &Settings, mode: &Mode, app: &mut crate::model::App) -> Vec<Action> {
    if mode != &Mode::Navigation {
        return Vec::new();
    }

    let (current_vp, current_buffer) =
        app::get_focused_current_mut(&mut app.window, &mut app.contents);
    let selected = match current_buffer {
        Buffer::Directory(buffer) => model::get_selected_path(buffer, &current_vp.cursor),
        _ => None,
    };

    if let Some(selected) = selected {
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
