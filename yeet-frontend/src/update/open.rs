use yeet_buffer::model::Mode;
use yeet_keymap::message::QuitMode;

use crate::{
    action::Action,
    model::{FileTreeBuffer, FileTreeBufferSectionBuffer},
    settings::Settings,
};

use super::selection::get_selected_path_with_base;

pub fn selected(settings: &Settings, mode: &Mode, buffer: &mut FileTreeBuffer) -> Vec<Action> {
    if mode != &Mode::Navigation {
        return Vec::new();
    }

    if let Some(selected) = match &buffer.parent {
        FileTreeBufferSectionBuffer::Text(path, text_buffer) => get_selected_path_with_base(
            path.as_path(),
            text_buffer,
            Some(&buffer.parent_cursor),
            |path| path.exists(),
        ),
        _ => None,
    } {
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
