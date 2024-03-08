use std::path::PathBuf;

use yeet_keymap::message::{Buffer, Mode};

use crate::{
    action::Action,
    model::{
        buffer::{undo::BufferChanged, BufferLine, BufferResult},
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

        let mut actions = Vec::new();
        if let BufferResult::Changes(modifications) = result {
            let mut trashes = Vec::new();
            for modification in crate::model::buffer::undo::consolidate(&modifications) {
                match modification {
                    BufferChanged::LineAdded(_, name) => {
                        actions.push(Action::Task(Task::AddPath(path.join(name))))
                    }
                    BufferChanged::LineRemoved(_, name) => {
                        trashes.push(path.join(name));
                    }
                    // TODO: new_name is empty, add to consolidated Trash operation
                    BufferChanged::Content(_, old_name, new_name) => actions.push(Action::Task(
                        Task::RenamePath(path.join(old_name), path.join(new_name)),
                    )),
                }
            }

            if !trashes.is_empty() {
                let (transaction, obsolete) = model.junk.trash(trashes);
                for entry in transaction.entries {
                    actions.push(Action::Task(Task::TrashPath(entry)));
                }

                if let Some(obsolete) = obsolete {
                    for entry in obsolete.entries {
                        actions.push(Action::Task(Task::DeleteRegisterEntry(entry)));
                    }
                }
            }
        }

        actions
    } else {
        vec![]
    }
}

pub fn selection(model: &Model) -> Option<PathBuf> {
    let buffer = &model.current.buffer;
    if buffer.lines.is_empty() {
        return None;
    }

    let cursor = &buffer.cursor.as_ref()?;
    let current = &buffer.lines.get(cursor.vertical_index)?;
    let target = model.current.path.join(&current.content);

    if target.exists() {
        Some(target)
    } else {
        None
    }
}

pub fn selected_bufferline(model: &mut Model) -> Option<&mut BufferLine> {
    let buffer = &mut model.current.buffer;
    if buffer.lines.is_empty() {
        return None;
    }

    let cursor = &buffer.cursor.as_ref()?;

    buffer.lines.get_mut(cursor.vertical_index)
}
