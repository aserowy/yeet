use yeet_keymap::message::Buffer;

use crate::{
    action::{Action, PostView},
    layout::AppLayout,
    model::{
        buffer::{undo::BufferChanged, BufferResult},
        Model,
    },
    task::Task,
};

use super::buffer;

pub fn update(model: &mut Model, layout: &AppLayout, message: Option<&Buffer>) {
    let buffer = &mut model.current.buffer;
    let layout = &layout.current;

    super::set_viewport_dimensions(&mut buffer.view_port, layout);

    if let Some(message) = message {
        buffer::update(&model.mode, buffer, message);
    } else {
        buffer::reset_view(buffer);
    }
}

pub fn save_changes(model: &mut Model) -> Option<Vec<Action>> {
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
                    BufferChanged::LineAdded(_, name) => tasks.push(Action::PostView(
                        PostView::Task(Task::AddPath(path.join(name))),
                    )),
                    BufferChanged::LineRemoved(_, name) => tasks.push(Action::PostView(
                        // TODO: handle soft delete with register task
                        PostView::Task(Task::TrashPath(path.join(name))),
                    )),
                    BufferChanged::Content(_, old_name, new_name) => tasks.push(Action::PostView(
                        PostView::Task(Task::RenamePath(path.join(old_name), path.join(new_name))),
                    )),
                }
            }
        }

        Some(tasks)
    } else {
        None
    }
}
