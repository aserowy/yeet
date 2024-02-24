use std::path::PathBuf;

use yeet_keymap::message::{Buffer, Mode};

use crate::{
    action::Action,
    model::{
        buffer::{undo::BufferChanged, BufferResult},
        Model,
    },
    settings::Settings,
    task::Task,
    update::buffer,
};

pub fn update(model: &mut Model, message: Option<&Buffer>) {
    let buffer = &mut model.current.buffer;
    let layout = &model.layout.current;

    super::set_viewport_dimensions(&mut buffer.view_port, layout);

    if let Some(message) = message {
        buffer::update(&model.mode, buffer, message);
    } else {
        buffer::reset_view(buffer);
    }
}

pub fn open(model: &Model, settings: &Settings) -> Option<Vec<Action>> {
    if model.mode != Mode::Navigation {
        return None;
    }

    if let Some(selected) = selection(model) {
        if settings.stdout_on_open {
            Some(vec![Action::Quit(Some(
                selected.to_string_lossy().to_string(),
            ))])
        } else {
            Some(vec![Action::Open(selected)])
        }
    } else {
        None
    }
}

pub fn save_changes(model: &mut Model) -> Vec<Action> {
    if let Some(result) = buffer::update(
        &model.mode,
        &mut model.current.buffer,
        &Buffer::SaveBuffer(None),
    ) {
        let path = &model.current.path;

        let mut tasks = Vec::new();
        if let BufferResult::Changes(modifications) = result {
            for modification in crate::model::buffer::undo::consolidate(&modifications) {
                match modification {
                    BufferChanged::LineAdded(_, name) => {
                        tasks.push(Action::Task(Task::AddPath(path.join(name))))
                    }
                    // TODO: multiple deletes should get consolidated into a single task/archive
                    // TODO: delete only inserts to " and 1-9 register
                    BufferChanged::LineRemoved(_, name) => {
                        let (entry, old_entry) = model.register.trash(&path.join(name));
                        tasks.push(Action::Task(Task::TrashPath(entry)));
                        if let Some(old_entry) = old_entry {
                            tasks.push(Action::Task(Task::DeleteRegisterEntry(old_entry)));
                        }
                    }
                    // TODO: new_name is empty, add to consolidated Trash operation
                    BufferChanged::Content(_, old_name, new_name) => tasks.push(Action::Task(
                        Task::RenamePath(path.join(old_name), path.join(new_name)),
                    )),
                }
            }
        }

        tasks
    } else {
        vec![]
    }
}

pub fn selection(model: &Model) -> Option<PathBuf> {
    let buffer = &model.current.buffer;
    if buffer.lines.is_empty() {
        return None;
    }

    if let Some(cursor) = &buffer.cursor {
        let current = &buffer.lines[cursor.vertical_index];
        let target = model.current.path.join(&current.content);

        if target.exists() {
            Some(target)
        } else {
            None
        }
    } else {
        None
    }
}
