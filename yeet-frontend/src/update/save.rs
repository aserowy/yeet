use yeet_buffer::{
    message::BufferMessage,
    model::{
        undo::{consolidate_modifications, BufferChanged},
        BufferResult, Mode,
    },
};

use crate::{
    action::Action,
    model::{junkyard::JunkYard, App, Buffer},
    task::Task,
};

use super::{app, junkyard::trash_to_junkyard};

#[tracing::instrument(skip(app))]
pub fn changes(app: &mut App, junk: &mut JunkYard, mode: &Mode) -> Vec<Action> {
    let (vp, buffer) = match app::get_focused_current_mut(app) {
        (vp, Buffer::Directory(it)) => (vp, it),
        (_vp, Buffer::Image(_)) => return Vec::new(),
        (_vp, Buffer::Content(_)) => return Vec::new(),
        (_vp, Buffer::PathReference(_)) => return Vec::new(),
        (_vp, Buffer::Empty) => return Vec::new(),
    };

    let selected_index = buffer.buffer.cursor.vertical_index;
    let selection = buffer
        .buffer
        .lines
        .get(selected_index)
        .map(|line| line.content.clone());

    let mut content: Vec<_> = buffer.buffer.lines.drain(..).collect();
    content.retain(|line| !line.content.is_empty());

    let message = BufferMessage::SetContent(content);
    yeet_buffer::update(
        Some(vp),
        mode,
        &mut buffer.buffer,
        std::slice::from_ref(&message),
    );

    if let Some(selection) = selection {
        let message = BufferMessage::SetCursorToLineContent(selection.to_stripped_string());
        yeet_buffer::update(
            Some(vp),
            mode,
            &mut buffer.buffer,
            std::slice::from_ref(&message),
        );
    }

    let message = BufferMessage::SaveBuffer;
    let result = yeet_buffer::update(
        Some(vp),
        mode,
        &mut buffer.buffer,
        std::slice::from_ref(&message),
    );

    let mut actions = Vec::new();
    for br in result {
        if let BufferResult::Changes(modifications) = br {
            let path = &buffer.path;
            let mut trashes = Vec::new();
            for modification in consolidate_modifications(&modifications) {
                match modification {
                    BufferChanged::LineAdded(_, name) => {
                        if !name.is_empty() {
                            actions.push(Action::Task(Task::AddPath(
                                path.join(name.to_stripped_string()),
                            )))
                        }
                    }
                    BufferChanged::LineRemoved(_, name) => {
                        trashes.push(path.join(name.to_stripped_string()));
                    }
                    BufferChanged::Content(_, old_name, new_name) => {
                        let task = if new_name.is_empty() {
                            Task::DeletePath(path.join(old_name.to_stripped_string()))
                        } else {
                            Task::RenamePath(
                                path.join(old_name.to_stripped_string()),
                                path.join(new_name.to_stripped_string()),
                            )
                        };
                        actions.push(Action::Task(task));
                    }
                }
            }

            if !trashes.is_empty() {
                let (transaction, obsolete) = trash_to_junkyard(junk, trashes);
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
