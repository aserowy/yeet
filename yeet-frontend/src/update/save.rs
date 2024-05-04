use yeet_buffer::{
    message::BufferMessage,
    model::{
        undo::{consolidate_modifications, BufferChanged},
        BufferResult,
    },
    update::update_buffer,
};

use crate::{action::Action, model::Model, task::Task};

use super::current::get_current_selected_bufferline;

#[tracing::instrument(skip(model))]
pub fn persist_path_changes(model: &mut Model) -> Vec<Action> {
    let selection = get_current_selected_bufferline(model).map(|line| line.content.clone());

    let mut content: Vec<_> = model.files.current.buffer.lines.drain(..).collect();
    content.retain(|line| !line.content.is_empty());

    update_buffer(
        &model.mode,
        &mut model.files.current.buffer,
        &BufferMessage::SetContent(content),
    );

    if let Some(selection) = selection {
        update_buffer(
            &model.mode,
            &mut model.files.current.buffer,
            &BufferMessage::SetCursorToLineContent(selection),
        );
    }

    let result = update_buffer(
        &model.mode,
        &mut model.files.current.buffer,
        &BufferMessage::SaveBuffer,
    );

    let mut actions = Vec::new();
    for br in result {
        if let BufferResult::Changes(modifications) = br {
            let path = &model.files.current.path;
            let mut trashes = Vec::new();
            for modification in consolidate_modifications(&modifications) {
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
    }
    actions
}
