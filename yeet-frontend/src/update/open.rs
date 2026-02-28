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

    let (parent_vp, _, _) = app::directory_viewports(app);
    let parent_cursor = parent_vp.cursor.clone();
    let (parent, _, _) = app::directory_buffers(app);
    let buffer = match parent {
        Buffer::Directory(it) => it,
        _ => return Vec::new(),
    };

    if let Some(selected) = model::get_selected_path_with_base(
        buffer.path.as_path(),
        &buffer.buffer,
        &parent_cursor,
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
