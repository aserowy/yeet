use std::path::PathBuf;

use yeet_buffer::{
    message::BufferMessage,
    model::{
        undo::{self, BufferChanged},
        BufferLine, BufferResult, Mode,
    },
    update,
};

use crate::{action::Action, model::Model, task::Task};

pub fn update(model: &mut Model, message: Option<&BufferMessage>) {
    let buffer = &mut model.file_buffer.current.buffer;
    let layout = &model.layout.current;

    super::set_viewport_dimensions(&mut buffer.view_port, layout);

    if let Some(message) = message {
        update::update(&model.mode, &model.search, buffer, message);
    } else {
        update::update(
            &model.mode,
            &model.search,
            buffer,
            &BufferMessage::ResetCursor,
        );
    }
}

pub fn open(model: &Model) -> Vec<Action> {
    if model.mode != Mode::Navigation {
        return Vec::new();
    }

    if let Some(selected) = selection(model) {
        if model.settings.stdout_on_open {
            vec![Action::Quit(Some(selected.to_string_lossy().to_string()))]
        } else {
            vec![Action::Open(selected)]
        }
    } else {
        Vec::new()
    }
}

#[tracing::instrument(skip(model))]
pub fn save_changes(model: &mut Model) -> Vec<Action> {
    let selection = selected_bufferline(model).map(|line| line.content.clone());

    let mut content: Vec<_> = model.file_buffer.current.buffer.lines.drain(..).collect();
    content.retain(|line| !line.content.is_empty());

    update::update(
        &model.mode,
        &model.search,
        &mut model.file_buffer.current.buffer,
        &BufferMessage::SetContent(content),
    );

    if let Some(selection) = selection {
        update::update(
            &model.mode,
            &model.search,
            &mut model.file_buffer.current.buffer,
            &BufferMessage::SetCursorToLineContent(selection),
        );
    }

    if let Some(result) = update::update(
        &model.mode,
        &model.search,
        &mut model.file_buffer.current.buffer,
        &BufferMessage::SaveBuffer,
    ) {
        let path = &model.file_buffer.current.path;

        let mut actions = Vec::new();
        if let BufferResult::Changes(modifications) = result {
            let mut trashes = Vec::new();
            for modification in undo::consolidate(&modifications) {
                match modification {
                    BufferChanged::LineAdded(_, name) => {
                        if !name.is_empty() {
                            actions.push(Action::Task(Task::AddPath(path.join(name))))
                        }
                    }
                    BufferChanged::LineRemoved(_, name) => {
                        trashes.push(path.join(name));
                    }
                    BufferChanged::Content(_, old_name, new_name) => {
                        let task = if new_name.is_empty() {
                            Task::DeletePath(path.join(old_name))
                        } else {
                            Task::RenamePath(path.join(old_name), path.join(new_name))
                        };
                        actions.push(Action::Task(task));
                    }
                }
            }

            if !trashes.is_empty() {
                let (transaction, obsolete) = model.junk.trash(trashes);
                for entry in transaction.entries {
                    actions.push(Action::Task(Task::TrashPath(entry)));
                }

                if let Some(obsolete) = obsolete {
                    for entry in obsolete.entries {
                        actions.push(Action::Task(Task::DeleteJunkYardEntry(entry)));
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
    let buffer = &model.file_buffer.current.buffer;
    if buffer.lines.is_empty() {
        return None;
    }

    let cursor = &buffer.cursor.as_ref()?;
    let current = &buffer.lines.get(cursor.vertical_index)?;
    if current.content.is_empty() {
        return None;
    }

    let target = model.file_buffer.current.path.join(&current.content);
    if target.exists() {
        Some(target)
    } else {
        None
    }
}

pub fn selected_bufferline(model: &mut Model) -> Option<&mut BufferLine> {
    let buffer = &mut model.file_buffer.current.buffer;
    if buffer.lines.is_empty() {
        return None;
    }

    let cursor = &buffer.cursor.as_ref()?;

    buffer.lines.get_mut(cursor.vertical_index)
}
