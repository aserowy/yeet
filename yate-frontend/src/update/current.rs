use yate_keymap::message::Message;

use crate::{
    event::PostRenderAction,
    layout::AppLayout,
    model::{buffer::BufferChanged, Model},
    task::Task,
};

use super::{buffer, path};

pub fn update(
    model: &mut Model,
    layout: &AppLayout,
    message: &Message,
) -> Option<Vec<PostRenderAction>> {
    let buffer = &mut model.current_directory;
    let layout = &layout.current_directory;

    super::set_viewport_dimensions(&mut buffer.view_port, layout);
    buffer.lines = path::get_directory_content(&model.current_path);

    if let Some(modifications) = buffer::update(buffer, message) {
        let path = &model.current_path;

        let mut tasks = Vec::new();
        for modification in modifications {
            match modification {
                BufferChanged::LineDeleted(_, name) => {
                    tasks.push(PostRenderAction::Task(Task::DeleteFile(path.join(name))))
                }
            }
        }

        Some(tasks)
    } else {
        None
    }
}
